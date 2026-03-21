# Tachibana Securities e-Shiten API Reference (v4r8)

Version: v4.8-000 (released 2025-09-27, v4r7 retired 2025-11-29)
Source: `mfds_json_api_menu.html` (Shift-JIS decoded), `api_overview_v4r7.txt`, existing codebase

---

## 1. Overview

The Tachibana Securities e-Shiten API is a free API for JP stock trading, real-time market data,
and news retrieval. It provides programmatic access to the e-Shiten brokerage system via HTTPS.

### 1.1 Interfaces

The API comprises 3 logical interface types with 5 virtual URL endpoints:

| # | Interface | Protocol | Data Format | Encoding | Purpose |
|---|-----------|----------|-------------|----------|---------|
| 1 | Auth I/F | HTTPS GET or POST (v4r8+) | JSON | Shift-JIS | Login/Logout |
| 2 | REQUEST I/F | HTTPS GET or POST (v4r8+) | JSON | Shift-JIS | Orders, queries, master data, market prices |
| 3 | EVENT I/F (HTTP) | HTTPS GET (Chunk Response) | Proprietary | Shift-JIS | Real-time push notifications |
| 4 | EVENT I/F (WebSocket) | HTTPS 1.1 Upgrade + WebSocket | Proprietary (same as #3) | Shift-JIS | Real-time push notifications (v4r7+) |

### 1.2 Key Characteristics

- **JP market only** (TSE)
- **Rate limit**: 10 requests/second per customer
- **Session model**: Virtual URLs issued at login; 1 customer = 1 set of virtual URLs
- **Request-response mode**: REQUEST I/F is strictly serial (one-at-a-time); no parallel requests except master data download
- **EVENT I/F**: Push-based, always-connected; 1 connection per customer (HTTP chunk or WebSocket, whichever connected last wins)
- **Response compression**: Apache mod_deflate for `application/json` and `text/json` (since v3)

### 1.3 v4r8 Changes

The only change in v4r8 is the addition of **HTTPS POST** support for Auth I/F and REQUEST I/F.
- GET continues to work as before
- EVENT I/F remains GET-only (proprietary format)
- No functional changes to request/response data

### 1.4 Base URL

```
Production: https://kabuka.e-shiten.jp/e_api_v4r8/
Demo:       https://demo.e-shiten.jp/e_api_v4r8/  (estimated)
```

---

## 2. Authentication (Auth I/F)

### 2.1 Login

**Endpoint**: `https://kabuka.e-shiten.jp/e_api_v4r8/auth/`

**Method**: HTTPS GET (URL-encoded JSON query string) or HTTPS POST (v4r8+)

#### Request Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `sUserId` | string | Yes | e-Shiten login ID |
| `sPassword` | string | Yes | Login password (URL-encode special chars: `# + / : =`) |
| `sSecondPassword` | string | Yes | Second password (required for all order operations) |
| `p_no` | string | Yes | Request sequence number (monotonically increasing) |
| `p_sd_date` | string | Yes | Client timestamp `YYYY.MM.DD-HH:MM:SS.mmm` |

#### Response Fields

| Field | Type | Description |
|-------|------|-------------|
| `p_errno` | string | Error number (`"0"` = success) |
| `p_err` | string | Error text (when `p_errno != "0"`) |
| `sUrlRequest` | string | Virtual URL for business functions (REQUEST I/F) |
| `sUrlMaster` | string | Virtual URL for master data functions (REQUEST I/F) |
| `sUrlPrice` | string | Virtual URL for market price functions (REQUEST I/F) |
| `sUrlEvent` | string | Virtual URL for EVENT I/F (HTTP Chunk version) |
| `sUrlEventWebSocket` | string | Virtual URL for EVENT I/F (WebSocket version, v4r7+) |
| `sKinsyouhouMidokuFlg` | string | `"1"` = unread financial documents exist (must read via web browser first) |
| `sKoufuSyomenUpdateYoteiDay` | string | Scheduled date for document updates (v4r5+) |
| `sApiReleaseYoteiDay` | string | Scheduled API release date (v4r5+) |

#### Authentication Notes

- If `sKinsyouhouMidokuFlg == "1"`, virtual URLs are not issued. User must log in via web browser and acknowledge documents first.
- **Phone verification** is required since 2025-07-26. On first authentication, phone verification must be completed. After that, the virtual URL remains valid until invalidated.
- Virtual URLs remain valid until explicitly invalidated; they can be reused across multiple sessions without re-authenticating each time.

### 2.2 Logout

Send a REQUEST I/F command with `sCLMID = "CLMLogout"` to the virtual URL (REQUEST).

#### Request

```json
{
  "sCLMID": "CLMLogout",
  "p_no": "N",
  "p_sd_date": "YYYY.MM.DD-HH:MM:SS.mmm"
}
```

### 2.3 Virtual URL Lifecycle

Virtual URLs are invalidated by any of the following:

| Event | Effect |
|-------|--------|
| Logout | All virtual URLs invalidated |
| Re-authentication (same user ID) | Previous virtual URLs invalidated |
| System closure (03:30 JST daily) | All virtual URLs invalidated |

After invalidation, any request to the old virtual URL returns an error response.

Virtual URLs are opaque strings with no fixed format (they change arbitrarily).

---

## 3. REQUEST I/F

### 3.1 URL Construction (GET)

For GET requests, the JSON payload is URL-encoded and appended as a query string:

```
{virtual_url}?{url_encoded_json}
```

Example:
```
https://virtual-url-string/?%7B%22sCLMID%22%3A%22CLMKabuNewOrder%22%2C...%7D
```

Characters that require URL encoding: `# + / : =` and all non-ASCII (Shift-JIS encoded).

### 3.2 POST Support (v4r8+)

v4r8 adds HTTP POST as an alternative. The JSON payload is sent in the request body instead of the URL query string. The response format is identical to GET.

### 3.3 Request Common Fields

Every REQUEST I/F request must include:

| Field | Type | Description |
|-------|------|-------------|
| `sCLMID` | string | Command identifier (e.g., `"CLMKabuNewOrder"`) |
| `p_no` | string | Request sequence number. Must be strictly increasing per session. Server rejects if `current p_no <= previous p_no` |
| `p_sd_date` | string | Client timestamp `YYYY.MM.DD-HH:MM:SS.mmm`. Server rejects if >30s behind server time (`p_errno=8`) |

Optional:

| Field | Type | Description |
|-------|------|-------------|
| `sJsonOfmt` | string | Response format specifier |

#### sJsonOfmt Values

| Value | Description |
|-------|-------------|
| `"0"` or `"2"` | Field numbers, no newlines |
| `"1"` or `"3"` | Field numbers, with newlines |
| `"4"` | Field names, no newlines (v4r1+) |
| `"5"` | Field names, with newlines (v4r1+) |

### 3.4 Response Common Fields

| Field | Type | Description |
|-------|------|-------------|
| `p_errno` | string | Request-level error (`"0"` = OK) |
| `p_err` | string | Error message text (present when `p_errno != "0"`) |
| `sResultCode` | string | Business-level result (`"0"` = OK) |
| `sResultText` | string | Result message text |

### 3.5 Virtual URL Routing

Different virtual URLs serve different function groups:

| Virtual URL | Functions | Parallel Use |
|-------------|-----------|--------------|
| `sUrlRequest` | Orders, queries, account info | Serial only (one-at-a-time) |
| `sUrlMaster` | Master data download, master data query, news | Download: parallel OK; Query: serial |
| `sUrlPrice` | Market prices, historical data | Serial only |
| `sUrlEvent` / `sUrlEventWebSocket` | Push notifications | 1 connection only |

All virtual URLs can be used in parallel *across groups* (e.g., REQUEST + EVENT simultaneously).
Within REQUEST I/F, requests must be strictly serial (wait for response before sending next request).

---

## 4. Order Operations

### 4.1 CLMKabuNewOrder (New Stock Order)

**sCLMID**: `"CLMKabuNewOrder"`
**Virtual URL**: `sUrlRequest`

#### Request Parameters

| Parameter | Type | Required | Description | Values |
|-----------|------|----------|-------------|--------|
| `sCLMID` | string | Yes | `"CLMKabuNewOrder"` | |
| `sIssueCode` | string | Yes | Stock ticker code | e.g., `"7203"` |
| `sOrderSizyouC` | string | Yes | Market code | `"00"` = TSE |
| `sOrderBaibaiKubun` | string | Yes | Buy/Sell | `"1"` = Sell, `"3"` = Buy |
| `sGenkinSinyouKubun` | string | Yes | Cash/Margin | `"0"` = Cash (spot), `"2"` = Margin (new position) |
| `sOrderCondition` | string | Yes | Order condition | `"0"` = Normal, `"1"` = OCO, etc. |
| `sOrderOrderPriceKubun` | string | Yes | Price type | `"1"` = Market, `"2"` = Limit |
| `sOrderOrderPrice` | string | Yes* | Order price | Required for limit orders; empty for market orders |
| `sOrderOrderSuryou` | string | Yes | Order quantity (shares) | e.g., `"100"` |
| `sOrderOrderExpireDay` | string | Yes | Expiration | `"0"` = Day only, `"YYYYMMDD"` = specific date |
| `sGyousyaCode` | string | No | Broker code | Usually empty |
| `sOrderTatebiType` | string | No | Position date type (margin) | |
| `sOrderTategyokuNumber` | string | No | Position number (margin close) | |
| `p_no` | string | Yes | Request sequence number | |
| `p_sd_date` | string | Yes | Client timestamp | |

#### sGenkinSinyouKubun Values

| Value | Description |
|-------|-------------|
| `"0"` | Cash (spot) trading |
| `"2"` | Margin: new position (system margin) |
| `"4"` | Margin: new position (general margin) |
| `"6"` | Margin: new position (day trade margin) |

#### sOrderCondition Values

| Value | Description |
|-------|-------------|
| `"0"` | Normal order |
| `"1"` | Stop order (reverse-limit / 逆指値) |
| `"2"` | Normal + Stop order combination |

#### sOrderOrderExpireDay Values

| Value | Description |
|-------|-------------|
| `"0"` | Day order (valid today only) |
| `"YYYYMMDD"` | GTC until specified date |

#### sOrderBaibaiKubun Values

| Value | Description |
|-------|-------------|
| `"1"` | Sell |
| `"3"` | Buy |

#### sOrderSizyouC Values (Market Code)

| Value | Description |
|-------|-------------|
| `"00"` | TSE (Tokyo Stock Exchange) |

#### Response Fields

| Field | Type | Description |
|-------|------|-------------|
| `p_errno` | string | `"0"` = success |
| `sResultCode` | string | `"0"` = success |
| `sResultText` | string | Result message |
| `sOrderNumber` | string | Assigned order number |

### 4.2 CLMKabuCorrectOrder (Order Correction)

**sCLMID**: `"CLMKabuCorrectOrder"`

Corrects (amends) an existing open order. Requires the second password.

#### Key Parameters

| Parameter | Description |
|-----------|-------------|
| `sCLMID` | `"CLMKabuCorrectOrder"` |
| `sOrderNumber` | Order number to correct |
| `sOrderOrderPrice` | New price |
| `sOrderOrderSuryou` | New quantity |
| `sOrderOrderExpireDay` | New expiration |
| `p_no` | Request sequence number |
| `p_sd_date` | Client timestamp |

### 4.3 CLMKabuCancelOrder (Order Cancellation)

**sCLMID**: `"CLMKabuCancelOrder"`

Cancels an existing open order. Also supports batch cancellation (v4r2+).

#### Key Parameters

| Parameter | Description |
|-----------|-------------|
| `sCLMID` | `"CLMKabuCancelOrder"` |
| `sOrderNumber` | Order number to cancel (or empty for batch cancel) |
| `p_no` | Request sequence number |
| `p_sd_date` | Client timestamp |

#### Batch Cancel

When `sOrderNumber` is empty or a special batch identifier is used, all open orders are cancelled.
Added in v4r2 (2022-09-03).

### 4.4 CLMOrderList (Order List Query)

**sCLMID**: `"CLMOrderList"`

Retrieves a list of current orders.

#### Response

Returns an array of order summary records.

### 4.5 CLMOrderListDetail (Order Detail Query)

**sCLMID**: `"CLMOrderListDetail"`
**Virtual URL**: `sUrlRequest`

#### Request Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `sCLMID` | string | Yes | `"CLMOrderListDetail"` |
| `sOrderNumber` | string | Yes | Order number to query |
| `sEigyouDay` | string | No | Business date `YYYYMMDD` (empty = today) |
| `p_no` | string | Yes | Request sequence number |
| `p_sd_date` | string | Yes | Client timestamp |

#### Response Fields

| Field | Type | Description |
|-------|------|-------------|
| `sOrderNumber` | string | Order number |
| `sIssueCode` | string | Stock ticker |
| `sOrderSizyouC` | string | Market code (`"00"` = TSE) |
| `sOrderBaibaiKubun` | string | Buy/Sell (`"1"` = Sell, `"3"` = Buy) |
| `sGenkinSinyouKubun` | string | Cash/Margin (`"0"` = Cash) |
| `sOrderOrderPriceKubun` | string | Price type (`"1"` = Market, `"2"` = Limit) |
| `sOrderOrderPrice` | string | Order price |
| `sOrderOrderSuryou` | string | Order quantity |
| `sOrderCurrentSuryou` | string | Effective (remaining) quantity |
| `sOrderStatusCode` | string | Order status code (see section 8.2) |
| `sOrderOrderExpireDay` | string | Expiration date `YYYYMMDD` |
| `sYakuzyouPrice` | string | Fill price |
| `sYakuzyouSuryou` | string | Fill quantity |
| `sBaiBaiDaikin` | string | Trade amount |
| `sBaiBaiTesuryo` | string | Commission |
| `sShouhizei` | string | Consumption tax |
| `aYakuzyouSikkouList` | array | Fill/expiration detail list |

### 4.6 Fill Summary / Fill Detail Query

#### CLMYakuzyouSummary (Fill Summary)

**sCLMID**: `"CLMYakuzyouSummary"` (estimated)

#### CLMYakuzyouDetail (Fill Detail)

Queries fill (execution) details.

---

## 5. Account / Position Queries

### 5.1 Spot Holdings List (現物保有銘柄一覧)

**sCLMID**: `"CLMGenbutuHoyuuList"` (estimated)

Retrieves the list of currently held spot (cash) stock positions.

### 5.2 Margin Positions List (信用建玉一覧)

**sCLMID**: `"CLMShinyouTategyokuList"`

Retrieves margin (credit) position details. Important to check `sHensaiKanouSuryou` (repayable quantity) before placing close-out orders.

**Important**: After cancelling a margin close-out order, wait and check `CLMShinyouTategyokuList` before placing a new close-out order. Immediate re-submission may cause "信用建玉明細にデータなし" error (see FAQ Q9).

### 5.3 Buying Power (買余力)

**sCLMID**: `"CLMKaiYoryoku"` (estimated)

Retrieves current buying power (available cash for trading).

### 5.4 Margin Capacity & Maintenance Ratio (建余力＆本日維持率)

**sCLMID**: `"CLMTateYoryoku"` (estimated)

Returns margin capacity and today's maintenance ratio.

### 5.5 Sellable Quantity (売却可能数量)

**sCLMID**: `"CLMBaikyakuKanouSuryou"` (estimated)

Returns the quantity of shares available for sale.

### 5.6 Available Amount Summary (可能額サマリー)

**sCLMID**: `"CLMKanougakuSummary"` (estimated)

Returns a summary of available trading amounts.

**Important**: Do NOT poll this endpoint. Use EVENT I/F for fill notifications, then query account status only when triggered (see FAQ Q12).

### 5.7 Available Amount History (可能額推移)

**sCLMID**: `"CLMKanougakuSuii"` (estimated)

Returns the history/trend of available trading amounts.

### 5.8 Spot Stock Purchase Available Amount Detail (現物株式買付可能額詳細)

Detailed breakdown of available amount for spot stock purchases.

### 5.9 Margin New Position Available Amount Detail (信用新規建て可能額詳細)

Detailed breakdown of available amount for new margin positions.

### 5.10 Real-time Margin Rate (リアル保証金率)

Real-time margin guarantee rate.

---

## 6. Master Data (REQUEST I/F)

Master data is accessed via `sUrlMaster` virtual URL.

### 6.1 Master Data Download (マスタ情報ダウンロード)

A streaming download mechanism (not standard request-response). After the initial request, the server continuously sends records until download is complete, then sends update records as they occur during the trading day.

**Characteristics**:
- Takes approximately 40 seconds to complete initial download
- Can be run in parallel with other virtual URLs
- Multiple master downloads can run in parallel with each other
- Continues streaming until client disconnects or logs out

#### Available Master Data Types

| Master Type | Description |
|-------------|-------------|
| System Status | e-Shiten system operational status |
| Date Information | Business dates, holidays |
| Tick Size (呼値) | Price tick increments |
| Operation Status by State | System operation status details |
| Operation Status (Stocks) | Stock market operation status |
| Operation Status (Derivatives) | Derivative market operation status |
| Stock Issue Master (株式銘柄マスタ) | Stock ticker master data |
| Stock Issue-Market Master (株式銘柄市場マスタ) | Stock ticker per-market data |
| Stock Issue-Market Regulations (株式銘柄別・市場別規制) | Per-stock per-market trading restrictions |
| Futures Issue Master (先物銘柄マスタ) | Futures contract master |
| Options Issue Master (オプション銘柄マスタ) | Options contract master |
| Derivative Issue-Market Regulations (派生銘柄別・市場別規制) | Derivative trading restrictions |
| Substitute Collateral Rate (代用掛目) | Collateral valuation rates |
| Margin Master (保証金マスタ) | Margin requirement master |
| Exchange Error Codes (取引所エラー等理由コード) | Exchange error/reason codes |

#### Master Data Notes (v4r4+)

- Excluded from stock master download: OTC stocks (market code `09`)
- Excluded from futures/options: expired contracts (delivery month < business month)
- TSE-only for certain fields: `代用証券評価単価` (stock master), `値幅下限/上限/前日終値` (market master) - non-TSE markets return empty strings

### 6.2 Master Data Query (マスタ情報問合取得)

A request-response API that allows querying specific master data items. More efficient than full download when only specific data is needed.

**sCLMID**: `"CLMMfdsGetMasterData"` (v4r2+)

#### Available Query Targets

| Target | Description |
|--------|-------------|
| Date Information | Business dates |
| Stock Issue Master | Stock ticker data |
| Stock Issue-Market Master | Per-market stock data |
| Futures Issue Master | Futures data |
| Options Issue Master | Options data |
| Other (Index etc.) Issue Master | Index/other instrument data |
| Exchange Error Codes | Error reason codes |

### 6.3 News Header Query (ニュースヘッダー問合取得)

**sCLMID**: `"CLMMfdsGetNewsHead"` (v4r4+)

Retrieves news headline/header information.

### 6.4 News Body Query (ニュースボディー問合取得)

**sCLMID**: `"CLMMfdsGetNewsBody"` (v4r4+)

Retrieves full news article body text.

### 6.5 Issue Detail Query (銘柄詳細情報問合取得)

**sCLMID**: `"CLMMfdsGetIssueDetail"` (v4r6+)

Retrieves detailed information about a specific stock issue.

### 6.6 Securities Finance Balance Query (証金残情報問合取得)

**sCLMID**: `"CLMMfdsGetSyoukinZan"` (v4r6+)

Retrieves securities finance (日証金) balance information.

### 6.7 Margin Balance Query (信用残情報問合取得)

**sCLMID**: `"CLMMfdsGetShinyouZan"` (v4r6+)

Retrieves credit/margin trading balance information.

### 6.8 Reverse Daily Interest Query (逆日歩情報問合取得)

**sCLMID**: `"CLMMfdsGetHibuInfo"` (v4r6+)

Retrieves reverse daily interest (逆日歩 / premium charge for short selling) information.

### 6.9 Market Price Query (時価情報問合取得)

**sCLMID**: `"CLMMfdsGetMarketPrice"` (v4r2+)
**Virtual URL**: `sUrlPrice`

Retrieves current market prices for specified stocks. Supports up to **120 stock codes** per request.

Available data items correspond to the PC stock board display fields.

### 6.10 Historical Price Query (蓄積情報問合取得)

**sCLMID**: `"CLMMfdsGetMarketPriceHistory"` (v4r3+)
**Virtual URL**: `sUrlPrice`

Retrieves historical OHLCV data for a specified stock. Data goes back approximately 20 years.

Returns:
- OHLCV (with and without stock split adjustment)
- Stock split dates with pre/post unit counts and conversion factors

---

## 7. EVENT I/F

### 7.1 Overview

EVENT I/F is a push-based interface for receiving real-time notifications. Two protocol versions exist:

| Version | Protocol | Added In | Status |
|---------|----------|----------|--------|
| HTTP Chunk | HTTPS GET with chunked transfer encoding | v1 | Active |
| WebSocket | HTTPS 1.1 Upgrade + WebSocket Protocol (RFC 6455) | v4r7 | Active |

Both versions deliver the same data in the same proprietary format.

**Important constraints**:
- Only 1 EVENT I/F connection per customer
- If both HTTP Chunk and WebSocket are connected, the later connection wins (earlier one is disconnected)
- If a new EVENT request is sent while one is active, the old connection is silently dropped (session is NOT invalidated)

### 7.2 Connection

#### HTTP Chunk Version
Connect to `sUrlEvent` with an HTTPS GET request. The server responds with a chunked transfer encoding response that remains open, sending data chunks as events occur.

#### WebSocket Version (v4r7+)
Connect to `sUrlEventWebSocket` using WebSocket protocol:

```javascript
var ws = new WebSocket(sUrlEventWebSocket);
```

After the WebSocket handshake (HTTP Upgrade), the server pushes event notifications via WebSocket frames.

### 7.3 Event Types (Notification Categories)

Events are identified by the `p_evt_cmd` field:

| p_evt_cmd | Type | Description |
|-----------|------|-------------|
| `EC` | Execution | Order/execution notifications (order status changes, fills, rejections) |
| `ST` | Status | System status notifications (open, close, etc.) |
| `KP` | Market Price | Real-time stock prices (throttled) |
| `FD` | Tick Data | Real-time tick/quote data (配信指定) |
| `NS` | News | Real-time news notifications |
| `SS` | Session Status | Session status changes |
| `US` | User Status | User-specific status changes |

### 7.4 EC (Execution) Event Details

EC events are the primary mechanism for detecting order fills.

#### Key Fields in EC Notifications

| Field | Type | Description |
|-------|------|-------------|
| `p_evt_cmd` | string | `"EC"` |
| `sOrderNumber` | string | Order number |
| `sIssueCode` | string | Stock ticker |
| `sOrderStatusCode` | string | Order status code (see section 8.2) |
| `sYakuzyouPrice` | string | Fill price (when filled) |
| `sYakuzyouSuryou` | string | Fill quantity (when filled) |

#### EC Event Sequence for a Spot Buy Order

```
1. Order state change notification (order submitted)
2. Order accepted (or acceptance error)
3. (Market processing)
4. Fill notification (sOrderStatusCode = "10" for full, "9" for partial)
```

**Timing warning**: REQUEST I/F response and EVENT I/F notifications are on separate connections. The EVENT I/F notification may arrive before or after the REQUEST I/F response depending on network conditions and client processing.

**Order warning**: Within EVENT I/F, notifications are delivered in the order the server sent them (streaming). However, the Tachibana business system may send correction/cancellation notifications out of order (e.g., "correction complete" before "correction accepted").

### 7.5 Subscription

To subscribe to specific event types, send a registration message:

```json
{
  "p_evt_cmd": "EC",
  "p_no": "N",
  "p_sd_date": "YYYY.MM.DD-HH:MM:SS.mmm"
}
```

### 7.6 ST (Status) Event

System status notifications include `p_errno` and `p_err` fields (v4r7+ change: `p_errno`/`p_err` are now only included in ST notifications, not all event types).

### 7.7 Market Price Throttling

Real-time market prices via EVENT I/F are subject to throttling (間引き処理). Prices may be delayed depending on the client's network conditions. This is a best-effort delivery mechanism.

### 7.8 WebSocket Service Note

If server load becomes problematic due to WebSocket connections, Tachibana may immediately suspend the WebSocket service. HTTP Chunk EVENT I/F will continue to function. Applications should be designed to fall back to HTTP Chunk if WebSocket becomes unavailable.

---

## 8. Error Handling

### 8.1 p_errno (Request-Level Errors)

| p_errno | Description |
|---------|-------------|
| `0` | Success |
| `-2` | Parameter error |
| `2` | Virtual URL invalid (session expired, logged out, or re-authenticated) |
| `8` | Time synchronization error (`p_sd_date` >30s behind server time) |
| Other non-zero | System error (e-Shiten system down, closed, or API subsystem failure) |

When `p_errno != 0`, the `p_err` field contains the error message text.

### 8.2 sOrderStatusCode (Order Status Codes)

| Code | Japanese | English | kekekabu Mapping |
|------|----------|---------|-----------------|
| `0` | 受付未済 | Pending Receipt | `pending` |
| `1` | 未約定 | Open (Unfilled) | `pending` |
| `2` | 受付エラー | Rejected | `rejected` |
| `3` | 訂正中 | Modifying | (transient) |
| `4` | 訂正完了 | Modified | (transient) |
| `5` | 訂正失敗 | Modify Failed | (transient) |
| `6` | 取消中 | Cancelling | (transient) |
| `7` | 取消完了 | Cancelled | `cancelled` |
| `8` | 取消失敗 | Cancel Failed | (transient) |
| `9` | 一部約定 | Partial Fill | `partial` |
| `10` | 全部約定 | Filled | `filled` |
| `11` | 一部失効 | Partial Expired | (transient) |
| `12` | 全部失効 | Expired | `expired` |
| `13` | 発注待ち | Queued | `pending` |
| `14` | 無効 | Invalid | (error) |
| `15` | 切替注文 / 逆指注文(切替中) | Switching | (transient) |
| `16` | 切替完了 / 逆指注文(未約定) | Switch Done | (transient) |
| `17` | 切替注文失敗 / 逆指注文(失敗) | Switch Failed | (transient) |
| `19` | 繰越失効 | Carry-Over Expired | `expired` |
| `20` | 一部障害処理 | Partial Error | (error) |
| `21` | 障害処理 | Error | (error) |
| `50` | 発注中 (逆指値) | Submitting (stop order) | (transient) |

### 8.3 sResultCode (Business Result Codes)

| sResultCode | Description |
|-------------|-------------|
| `"0"` | Success |
| Non-zero | Business error (details in `sResultText`) |

### 8.4 Warning Codes

Phone verification related warnings were added in v4r7. Refer to the manual section 7 (結果コード、警告コード表) for the complete list.

---

## 9. Important Notes

### 9.1 Rate Limits

- **10 requests/second** per customer, enforced server-side
- Excessive load may result in account suspension

### 9.2 Operating Hours

- Available during e-Shiten system operating hours
- System closure at approximately 03:30 JST daily
- Maintenance and holidays follow e-Shiten standard web schedule
- If the API subsystem fails but e-Shiten is operational, use the standard web interface for order management

### 9.3 Encoding

- **Request**: JSON with Shift-JIS encoding for Japanese characters. When using GET, the JSON is URL-encoded.
- **Response**: JSON in Shift-JIS encoding. Must decode from Shift-JIS to UTF-8 in client code.
- Response compression: gzip via Apache mod_deflate for `application/json` and `text/json` content types.

### 9.4 URL Encoding Requirements

Characters with special meaning in URLs must be percent-encoded:

| Character | Encoded |
|-----------|---------|
| `#` | `%23` |
| `+` | `%2B` |
| `/` | `%2F` |
| `:` | `%3A` |
| `=` | `%3D` |

This is particularly important for passwords that may contain these characters.

### 9.5 Serial Request Requirement

REQUEST I/F operates in a strict serial (one-at-a-time) mode:
- Send a request, wait for the response, then send the next request
- Parallel requests to the same virtual URL result in undefined behavior
- **Exception**: Master data download via `sUrlMaster` can run in parallel

### 9.6 Duplicate Request Prevention (p_no)

The `p_no` field prevents duplicate request processing:
- Initialize at login time
- Increment by 1 or more for each subsequent request
- Server rejects requests where `p_no <= previous p_no`
- This guards against browser/client automatic retry sending duplicate orders

### 9.7 Server Fault Tolerance

- API servers run on multiple parallel instances
- If one server fails, traffic is routed to surviving servers
- Requests in-flight during a server crash will not receive an API response (HTTPS-level error)
- After such failures, check order status via `CLMOrderListDetail` or the standard web interface

### 9.8 Insider Trading Restriction

Stocks declared for insider trading restrictions cannot be traded via API for new/correction orders.
Cancellation orders ARE permitted for insider-declared stocks (emergency consideration).
Use the standard web interface for new/correction orders on insider-declared stocks.

### 9.9 Second Password Requirement

The second password (`sSecondPassword`) is required for all order input operations (new, correct, cancel) regardless of the "password omission" setting in the standard web interface.

### 9.10 v4r8 POST Compatibility

When migrating from v4r7 to v4r8:
- Simply change the URL prefix from `e_api_v4r7` to `e_api_v4r8`
- Existing GET-based code continues to work without changes
- POST support is optional; use it when URL length limits are a concern

---

## 10. Provided Functions Summary

### 10.1 Auth Functions (Auth I/F)

| Function | Description |
|----------|-------------|
| Login | Authenticate and obtain virtual URLs |
| Logout | Invalidate virtual URLs |

### 10.2 Business Functions (REQUEST I/F via sUrlRequest)

| sCLMID | Function |
|--------|----------|
| `CLMKabuNewOrder` | New stock order |
| `CLMKabuCorrectOrder` | Order correction/amendment |
| `CLMKabuCancelOrder` | Order cancellation (including batch cancel) |
| `CLMGenbutuHoyuuList` (*) | Spot holdings list |
| `CLMShinyouTategyokuList` | Margin positions list |
| `CLMKaiYoryoku` (*) | Buying power |
| `CLMTateYoryoku` (*) | Margin capacity & maintenance ratio |
| `CLMBaikyakuKanouSuryou` (*) | Sellable quantity |
| `CLMOrderList` | Order list |
| `CLMOrderListDetail` | Order/fill detail |
| `CLMKanougakuSummary` (*) | Available amount summary |
| `CLMKanougakuSuii` (*) | Available amount history |
| (unknown sCLMID) | Spot stock purchase available amount detail |
| (unknown sCLMID) | Margin new position available amount detail |
| (unknown sCLMID) | Real-time margin rate |

(*) sCLMID values marked with asterisk are estimated from Japanese function names; exact sCLMID values should be confirmed against the detailed manual.

### 10.3 Master Functions (REQUEST I/F via sUrlMaster)

| sCLMID | Function |
|--------|----------|
| (streaming) | Master data download |
| `CLMMfdsGetMasterData` | Master data query (v4r2+) |
| `CLMMfdsGetNewsHead` | News header query (v4r4+) |
| `CLMMfdsGetNewsBody` | News body query (v4r4+) |
| `CLMMfdsGetIssueDetail` | Issue detail query (v4r6+) |
| `CLMMfdsGetSyoukinZan` | Securities finance balance (v4r6+) |
| `CLMMfdsGetShinyouZan` | Margin balance query (v4r6+) |
| `CLMMfdsGetHibuInfo` | Reverse daily interest (v4r6+) |

### 10.4 Market Price Functions (REQUEST I/F via sUrlPrice)

| sCLMID | Function |
|--------|----------|
| `CLMMfdsGetMarketPrice` | Market price query (max 120 tickers, v4r2+) |
| `CLMMfdsGetMarketPriceHistory` | Historical OHLCV query (~20 years, v4r3+) |

### 10.5 Event Functions (EVENT I/F via sUrlEvent or sUrlEventWebSocket)

| Event Type | Description |
|------------|-------------|
| Order/Execution notifications | Order status changes, fills |
| System status | Open/close notifications |
| Operation status | Order acceptance start/end |
| Real-time market prices | Stock prices (throttled) |
| Real-time news | News delivery |

---

## 11. Version History

| Version | Date | Key Changes |
|---------|------|-------------|
| v4r8 | 2025-09-27 | HTTPS POST support for Auth I/F and REQUEST I/F |
| v4r7 | 2025-05-31 | WebSocket EVENT I/F; EVENT common field changes (p_errno only in ST) |
| v4r6 | 2025-02-22 | Issue detail, securities finance, margin balance, reverse daily interest queries |
| v4r5 | 2023-12-31 | New NISA (growth) support; manual page renewal |
| v4r4 | 2023-07-22 | News query I/F; master download filtering |
| v4r3 | 2022-11-19 | Virtual URL (MASTER/PRICE); historical price query; market price limit 120 |
| v4r2 | 2021-06-27 | p_sd_date delay check fix; market price query; master data query |
| v4r1 | 2021-02-13 | p_sd_date delay check; sJsonOfmt field name output |
| v4 | 2020-10-24 | p_no duplicate check (replacing p_sd_date) |
| v3 | 2020-08-21 | Response compression (mod_deflate) |

---

## Appendix A: Sample Request/Response

### Login Request (GET)

```
GET https://kabuka.e-shiten.jp/e_api_v4r8/auth/?%7B%22p_no%22%3A%221%22%2C%22p_sd_date%22%3A%222026.03.21-09%3A00%3A00.000%22%2C%22sUserId%22%3A%22MYID%22%2C%22sPassword%22%3A%22MYPASS%22%2C%22sSecondPassword%22%3A%22MY2NDPASS%22%7D
```

Decoded JSON:
```json
{
  "p_no": "1",
  "p_sd_date": "2026.03.21-09:00:00.000",
  "sUserId": "MYID",
  "sPassword": "MYPASS",
  "sSecondPassword": "MY2NDPASS"
}
```

### Login Response

```json
{
  "p_errno": "0",
  "p_err": "",
  "sUrlRequest": "https://kabuka.e-shiten.jp/xxx/yyy/zzz/request/",
  "sUrlMaster": "https://kabuka.e-shiten.jp/xxx/yyy/zzz/master/",
  "sUrlPrice": "https://kabuka.e-shiten.jp/xxx/yyy/zzz/price/",
  "sUrlEvent": "https://kabuka.e-shiten.jp/xxx/yyy/zzz/event/",
  "sUrlEventWebSocket": "wss://kabuka.e-shiten.jp/xxx/yyy/zzz/event_ws/",
  "sKinsyouhouMidokuFlg": "0"
}
```

### New Order Request (CLMKabuNewOrder)

```json
{
  "sCLMID": "CLMKabuNewOrder",
  "sIssueCode": "7203",
  "sOrderSizyouC": "00",
  "sOrderBaibaiKubun": "3",
  "sGenkinSinyouKubun": "0",
  "sOrderCondition": "0",
  "sOrderOrderPriceKubun": "2",
  "sOrderOrderPrice": "2500",
  "sOrderOrderSuryou": "100",
  "sOrderOrderExpireDay": "0",
  "sGyousyaCode": "",
  "sOrderTatebiType": "",
  "sOrderTategyokuNumber": "",
  "p_no": "2",
  "p_sd_date": "2026.03.21-09:00:01.000"
}
```

### New Order Response

```json
{
  "p_errno": "0",
  "sResultCode": "0",
  "sOrderNumber": "ORD001",
  "sResultText": "OK"
}
```

### Order Detail Query Response

```json
{
  "p_errno": "0",
  "sResultCode": "0",
  "sOrderNumber": "ORD001",
  "sIssueCode": "7203",
  "sOrderStatusCode": "10",
  "sOrderBaibaiKubun": "3",
  "sOrderOrderPrice": "2500",
  "sOrderOrderSuryou": "100",
  "sYakuzyouPrice": "2500",
  "sYakuzyouSuryou": "100",
  "sBaiBaiDaikin": "250000",
  "sBaiBaiTesuryo": "0",
  "sShouhizei": "0"
}
```

### EC Fill Notification (EVENT I/F)

```json
{
  "p_evt_cmd": "EC",
  "sOrderNumber": "ORD001",
  "sIssueCode": "7203",
  "sOrderStatusCode": "10",
  "sYakuzyouPrice": "2500",
  "sYakuzyouSuryou": "100"
}
```
