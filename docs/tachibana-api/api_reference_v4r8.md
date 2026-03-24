# Tachibana Securities e-Shiten API Reference (v4r8)

Version: v4.8-000 (released 2025-09-27, v4r7 retired 2025-11-29)
Source: `mfds_json_api_menu.html` (Shift-JIS decoded), official API reference page snapshot, existing codebase

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

### 2.1 Login (CLMAuthLoginRequest / CLMAuthLoginAck)

**Endpoint**: `https://kabuka.e-shiten.jp/e_api_v4r8/auth/`

**Method**: HTTPS GET (URL-encoded JSON query string) or HTTPS POST (v4r8+)

#### Request (sCLMID: `CLMAuthLoginRequest`)

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `sCLMID` | string | Yes | `"CLMAuthLoginRequest"` |
| `sUserId` | string | Yes | e-Shiten login ID |
| `sPassword` | string | Yes | Login password (URL-encode special chars: `# + / : =`) |
| `p_no` | string | Yes | Request sequence number (monotonically increasing) |
| `p_sd_date` | string | Yes | Client timestamp `YYYY.MM.DD-HH:MM:SS.mmm` |

#### Response (sCLMID: `CLMAuthLoginAck`)

| Field | Type | Description |
|-------|------|-------------|
| `sCLMID` | string | `"CLMAuthLoginAck"` |
| `sResultCode` | string | Business result (`"0"` = success; non-zero = see result/warning code table) |
| `sResultText` | string | Result message text (empty on success) |
| `sZyoutoekiKazeiC` | string | Tax account type: `"1"` = Specific, `"3"` = General, `"5"` = NISA |
| `sSecondPasswordOmit` | string | Second password omission: `"0"` = required (fixed value for API) |
| `sLastLoginDate` | string | Last login datetime `YYYYMMDDHHMMSS` |
| `sSogoKouzaKubun` | string | Comprehensive account: `"0"` = not opened, `"1"` = opened |
| `sHogoAdukariKouzaKubun` | string | Safekeeping account: `"0"` / `"1"` |
| `sFurikaeKouzaKubun` | string | Transfer settlement account: `"0"` / `"1"` |
| `sGaikokuKouzaKubun` | string | Foreign account: `"0"` / `"1"` |
| `sMRFKouzaKubun` | string | MRF account: `"0"` / `"1"` |
| `sTokuteiKouzaKubunGenbutu` | string | Specific account (spot): `"0"` = general, `"1"` = specific (no withholding), `"2"` = specific (with withholding) |
| `sTokuteiKouzaKubunSinyou` | string | Specific account (margin): same as above |
| `sTokuteiKouzaKubunTousin` | string | Specific account (investment trust): same as above |
| `sTokuteiHaitouKouzaKubun` | string | Dividend specific account: `"0"` / `"1"` |
| `sTokuteiKanriKouzaKubun` | string | Specific management account: `"0"` / `"1"` |
| `sSinyouKouzaKubun` | string | Margin trading account: `"0"` / `"1"` |
| `sSakopKouzaKubun` | string | Futures/Options account: `"0"` / `"1"` |
| `sMMFKouzaKubun` | string | MMF account: `"0"` / `"1"` |
| `sTyukokufKouzaKubun` | string | China Fund account: `"0"` / `"1"` |
| `sKawaseKouzaKubun` | string | FX margin account: `"0"` / `"1"` |
| `sHikazeiKouzaKubun` | string | Tax-exempt (NISA) account: `"0"` / `"1"` |
| `sKinsyouhouMidokuFlg` | string | `"1"` = unread financial documents (API unusable until acknowledged via web); `"0"` = read |
| `sUrlRequest` | string | Virtual URL for business functions (REQUEST I/F) |
| `sUrlMaster` | string | Virtual URL for master data functions (REQUEST I/F) |
| `sUrlPrice` | string | Virtual URL for market price functions (REQUEST I/F) |
| `sUrlEvent` | string | Virtual URL for EVENT I/F (HTTP Chunk version) |
| `sUrlEventWebSocket` | string | Virtual URL for EVENT I/F (WebSocket version, v4r7+) |
| `sUpdateInformWebDocument` | string | Scheduled date for web document updates |
| `sUpdateInformAPISpecFunction` | string | Scheduled API release date |

#### Authentication Notes

- If `sKinsyouhouMidokuFlg == "1"`, virtual URLs are not issued (set to `""`). User must log in via web browser and acknowledge documents first.
- **Phone verification** is required since 2025-07-26. On first authentication, phone verification must be completed. After that, the virtual URL remains valid until invalidated.
- Virtual URLs remain valid until explicitly invalidated; they can be reused across multiple sessions without re-authenticating each time.
- The `sUpdateInformWebDocument` and `sUpdateInformAPISpecFunction` fields notify of upcoming changes. Check: `(scheduled_date >= today) AND (scheduled_date != previous_value)`.

### 2.2 Logout (CLMAuthLogoutRequest / CLMAuthLogoutAck)

#### Request (sCLMID: `CLMAuthLogoutRequest`)

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `sCLMID` | string | Yes | `"CLMAuthLogoutRequest"` |

**Note**: Logout is sent to the Auth I/F endpoint, not the REQUEST I/F virtual URL.

#### Response (sCLMID: `CLMAuthLogoutAck`)

| Field | Type | Description |
|-------|------|-------------|
| `sCLMID` | string | `"CLMAuthLogoutAck"` |
| `sResultCode` | string | See CLMAuthLoginAck.sResultCode |
| `sResultText` | string | See CLMAuthLoginAck.sResultText |

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
| `sWarningCode` | string | Warning code (`"0"` = no warning) |
| `sWarningText` | string | Warning message text |

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
| `sZyoutoekiKazeiC` | string | Yes | Tax account type | `"1"` = Specific, `"3"` = General, `"5"` = NISA (sell only after 2024), `"6"` = NISA Growth (from 2024) |
| `sIssueCode` | string | Yes | Stock ticker code | e.g., `"6501"` |
| `sSizyouC` | string | Yes | Market code | `"00"` = TSE |
| `sBaibaiKubun` | string | Yes | Buy/Sell | `"1"` = Sell, `"3"` = Buy, `"5"` = Cash delivery (Genwatashi), `"7"` = Cash receipt (Genbiki) |
| `sCondition` | string | Yes | Execution condition | `"0"` = None, `"2"` = Opening (Yoritsuki), `"4"` = Closing (Hike), `"6"` = Funari |
| `sOrderPrice` | string | Yes | Order price | `"*"` = unspecified, `"0"` = market, otherwise limit price |
| `sOrderSuryou` | string | Yes | Order quantity (shares) | e.g., `"100"` |
| `sGenkinShinyouKubun` | string | Yes | Cash/Margin type | `"0"` = Cash, `"2"` = New margin (system 6mo), `"4"` = Close margin (system 6mo), `"6"` = New margin (general 6mo), `"8"` = Close margin (general 6mo) |
| `sOrderExpireDay` | string | Yes | Order expiration | `"0"` = Day only, `"YYYYMMDD"` = GTC until date (max 10 business days) |
| `sGyakusasiOrderType` | string | Yes | Stop order type | `"0"` = Normal, `"1"` = Stop (reverse limit), `"2"` = Normal + Stop combination |
| `sGyakusasiZyouken` | string | Yes | Stop trigger condition | `"0"` = None, otherwise trigger price |
| `sGyakusasiPrice` | string | Yes | Stop order price | `"*"` = None, `"0"` = market, otherwise limit price |
| `sTatebiType` | string | Yes | Position date type (margin close) | `"*"` = N/A (spot or new), `"1"` = Individual, `"2"` = Date order, `"3"` = Profit order, `"4"` = Loss order |
| `sTategyokuZyoutoekiKazeiC` | string | Yes | Position tax account (for Genbiki/Genwatashi) | `"*"` = N/A, `"1"` = Specific, `"3"` = General |
| `sSecondPassword` | string | Yes | Second password (order password) | |
| `aCLMKabuHensaiData` | array | No | Repayment position list (for individual margin close) | See CLMKabuHensaiData below |
| `p_no` | string | Yes | Request sequence number | |
| `p_sd_date` | string | Yes | Client timestamp | |

#### Response Fields

| Field | Type | Description |
|-------|------|-------------|
| `sCLMID` | string | `"CLMKabuNewOrder"` |
| `sResultCode` | string | `"0"` = success |
| `sResultText` | string | Result message |
| `sWarningCode` | string | Warning code (`"0"` = no warning) |
| `sWarningText` | string | Warning text |
| `sOrderNumber` | string | Assigned order number (unique with business day) |
| `sEigyouDay` | string | Business day `YYYYMMDD` |
| `sOrderUkewatasiKingaku` | string | Settlement amount |
| `sOrderTesuryou` | string | Commission |
| `sOrderSyouhizei` | string | Consumption tax |
| `sKinri` | string | Interest rate (for margin; `"-"` for spot) |
| `sOrderDate` | string | Order datetime `YYYYMMDDHHMMSS` |

#### CLMKabuHensaiData (Repayment Data)

When closing margin positions with `sTatebiType = "1"` (individual specification), provide an array `aCLMKabuHensaiData` with the following fields per entry:

| Field | Description |
|-------|-------------|
| `sTategyokuNumber` | Position number (from `CLMShinyouTategyokuList.sOrderTategyokuNumber`) |
| `sTatebiZyuni` | Repayment order priority (ascending from 1) |
| `sOrderSuryou` | Repayment quantity (shares) |

#### Request Examples

**Spot buy (market, specific account)**:
```json
{
  "sCLMID": "CLMKabuNewOrder",
  "sZyoutoekiKazeiC": "1",
  "sIssueCode": "6658",
  "sSizyouC": "00",
  "sBaibaiKubun": "3",
  "sCondition": "0",
  "sOrderPrice": "0",
  "sOrderSuryou": "100",
  "sGenkinShinyouKubun": "0",
  "sOrderExpireDay": "0",
  "sGyakusasiOrderType": "0",
  "sGyakusasiZyouken": "0",
  "sGyakusasiPrice": "*",
  "sTatebiType": "*",
  "sTategyokuZyoutoekiKazeiC": "*",
  "sSecondPassword": "pswd"
}
```

**Spot sell (limit, specific account)**:
```json
{
  "sCLMID": "CLMKabuNewOrder",
  "sZyoutoekiKazeiC": "1",
  "sIssueCode": "6658",
  "sSizyouC": "00",
  "sBaibaiKubun": "1",
  "sCondition": "0",
  "sOrderPrice": "201",
  "sOrderSuryou": "100",
  "sGenkinShinyouKubun": "0",
  "sOrderExpireDay": "0",
  "sGyakusasiOrderType": "0",
  "sGyakusasiZyouken": "0",
  "sGyakusasiPrice": "*",
  "sTatebiType": "*",
  "sTategyokuZyoutoekiKazeiC": "*",
  "sSecondPassword": "pswd"
}
```

**Margin close with individual position specification**:
```json
{
  "sCLMID": "CLMKabuNewOrder",
  "sZyoutoekiKazeiC": "1",
  "sIssueCode": "4241",
  "sSizyouC": "00",
  "sBaibaiKubun": "1",
  "sCondition": "0",
  "sOrderPrice": "920",
  "sOrderSuryou": "200",
  "sGenkinShinyouKubun": "4",
  "sOrderExpireDay": "0",
  "sGyakusasiOrderType": "0",
  "sGyakusasiZyouken": "0",
  "sGyakusasiPrice": "*",
  "sTatebiType": "1",
  "sTategyokuZyoutoekiKazeiC": "*",
  "sSecondPassword": "pswd",
  "aCLMKabuHensaiData": [
    {"sTategyokuNumber": "202007220000402", "sTatebiZyuni": "1", "sOrderSuryou": "100"},
    {"sTategyokuNumber": "202007220001591", "sTatebiZyuni": "2", "sOrderSuryou": "100"}
  ]
}
```

**Stop order (buy when price reaches 460, order at 455)**:
```json
{
  "sCLMID": "CLMKabuNewOrder",
  "sZyoutoekiKazeiC": "1",
  "sIssueCode": "3632",
  "sSizyouC": "00",
  "sBaibaiKubun": "3",
  "sCondition": "0",
  "sOrderPrice": "*",
  "sOrderSuryou": "100",
  "sGenkinShinyouKubun": "0",
  "sOrderExpireDay": "0",
  "sGyakusasiOrderType": "1",
  "sGyakusasiZyouken": "460",
  "sGyakusasiPrice": "455",
  "sTatebiType": "*",
  "sTategyokuZyoutoekiKazeiC": "*",
  "sSecondPassword": "pswd"
}
```

### 4.2 CLMKabuCorrectOrder (Order Correction)

**sCLMID**: `"CLMKabuCorrectOrder"`
**Virtual URL**: `sUrlRequest`

Corrects (amends) an existing open order. Requires the second password.

#### Request Parameters

| Parameter | Type | Required | Description | Values |
|-----------|------|----------|-------------|--------|
| `sCLMID` | string | Yes | `"CLMKabuCorrectOrder"` | |
| `sOrderNumber` | string | Yes | Order number to correct | From CLMKabuNewOrder response |
| `sEigyouDay` | string | Yes | Business day | From CLMKabuNewOrder response |
| `sCondition` | string | Yes | Execution condition | `"*"` = no change, `"0"` = none, `"2"` = opening, `"4"` = closing, `"6"` = funari |
| `sOrderPrice` | string | Yes | Order price | `"*"` = no change, `"0"` = change to market, otherwise new limit price |
| `sOrderSuryou` | string | Yes | Order quantity | `"*"` = no change, otherwise new quantity (no increase allowed; include partial fills in count) |
| `sOrderExpireDay` | string | Yes | Order expiration | `"*"` = no change, `"0"` = day, `"YYYYMMDD"` = new date |
| `sGyakusasiZyouken` | string | Yes | Stop trigger condition | `"*"` = no change, otherwise new trigger price |
| `sGyakusasiPrice` | string | Yes | Stop order price | `"*"` = no change, `"0"` = market, otherwise new price |
| `sSecondPassword` | string | Yes | Second password | |
| `p_no` | string | Yes | Request sequence number | |
| `p_sd_date` | string | Yes | Client timestamp | |

**Note**: After stop trigger fires, stop condition/price cannot be corrected. Use normal price correction instead.

#### Response Fields

| Field | Type | Description |
|-------|------|-------------|
| `sCLMID` | string | `"CLMKabuCorrectOrder"` |
| `sResultCode` | string | See CLMKabuNewOrder |
| `sResultText` | string | See CLMKabuNewOrder |
| `sOrderNumber` | string | Echo of request value |
| `sEigyouDay` | string | Echo of request value |
| `sOrderUkewatasiKingaku` | string | Settlement amount |
| `sOrderTesuryou` | string | Commission |
| `sOrderSyouhizei` | string | Consumption tax |
| `sOrderDate` | string | Order datetime `YYYYMMDDHHMMSS` |

### 4.3 CLMKabuCancelOrder (Order Cancellation)

**sCLMID**: `"CLMKabuCancelOrder"`
**Virtual URL**: `sUrlRequest`

Cancels an existing open order.

#### Request Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `sCLMID` | string | Yes | `"CLMKabuCancelOrder"` |
| `sOrderNumber` | string | Yes | Order number to cancel (from CLMKabuNewOrder response) |
| `sEigyouDay` | string | Yes | Business day (from CLMKabuNewOrder response) |
| `sSecondPassword` | string | Yes | Second password |
| `p_no` | string | Yes | Request sequence number |
| `p_sd_date` | string | Yes | Client timestamp |

#### Response Fields

| Field | Type | Description |
|-------|------|-------------|
| `sCLMID` | string | `"CLMKabuCancelOrder"` |
| `sResultCode` | string | See CLMKabuNewOrder |
| `sResultText` | string | See CLMKabuNewOrder |
| `sOrderNumber` | string | Echo of request value |
| `sEigyouDay` | string | Echo of request value |
| `sOrderUkewatasiKingaku` | string | Settlement amount |
| `sOrderDate` | string | Order datetime `YYYYMMDDHHMMSS` |

### 4.4 CLMKabuCancelOrderAll (Cancel All Orders)

**sCLMID**: `"CLMKabuCancelOrderAll"`
**Virtual URL**: `sUrlRequest`

Cancels all open orders at once.

#### Request Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `sCLMID` | string | Yes | `"CLMKabuCancelOrderAll"` |
| `sSecondPassword` | string | Yes | Second password |
| `p_no` | string | Yes | Request sequence number |
| `p_sd_date` | string | Yes | Client timestamp |

#### Response Fields

| Field | Type | Description |
|-------|------|-------------|
| `sCLMID` | string | `"CLMKabuCancelOrderAll"` |
| `sResultCode` | string | See CLMKabuNewOrder |
| `sResultText` | string | See CLMKabuNewOrder |

### 4.5 CLMOrderList (Order List Query)

**sCLMID**: `"CLMOrderList"`
**Virtual URL**: `sUrlRequest`

Retrieves a list of current orders with optional filters.

#### Request Parameters

| Parameter | Type | Required | Description | Values |
|-----------|------|----------|-------------|--------|
| `sCLMID` | string | Yes | `"CLMOrderList"` | |
| `sIssueCode` | string | No | Stock ticker filter | e.g., `"8411"` or `""` for all |
| `sSikkouDay` | string | No | Execution date (business day) filter | `"YYYYMMDD"` or `""` for all |
| `sOrderSyoukaiStatus` | string | No | Order status filter | `""` = all, `"1"` = unfilled, `"2"` = fully filled, `"3"` = partial, `"4"` = correctable/cancellable, `"5"` = unfilled + partial |
| `p_no` | string | Yes | Request sequence number | |
| `p_sd_date` | string | Yes | Client timestamp | |

**Note**: All filter parameters (except sCLMID) are optional AND-condition filters. The `sSikkouDay` is the order execution scheduled date which changes after the evening day-change processing; it is NOT for retrieving past order history.

#### Response Fields

| Field | Type | Description |
|-------|------|-------------|
| `sCLMID` | string | `"CLMOrderList"` |
| `sResultCode` | string | See CLMKabuNewOrder |
| `sResultText` | string | See CLMKabuNewOrder |
| `sWarningCode` | string | Warning code |
| `sWarningText` | string | Warning text |
| `sIssueCode` | string | Echo of request value |
| `sSikkouDay` | string | Echo of request value |
| `sOrderSyoukaiStatus` | string | Echo of request value |
| `aOrderList` | array | Order list (see below); `""` if no data |

**aOrderList array item fields:**

| Field | Description |
|-------|-------------|
| `sOrderWarningCode` | Warning code |
| `sOrderWarningText` | Warning text |
| `sOrderOrderNumber` | Order number |
| `sOrderIssueCode` | Stock ticker |
| `sOrderSizyouC` | Market code |
| `sOrderZyoutoekiKazeiC` | Tax account type |
| `sGenkinSinyouKubun` | Cash/Margin type |
| `sOrderBensaiKubun` | Repayment type: `"00"` = none, `"26"` = system 6mo, `"29"` = system unlimited, `"36"` = general 6mo, `"39"` = general unlimited |
| `sOrderBaibaiKubun` | Buy/Sell |
| `sOrderOrderSuryou` | Order quantity |
| `sOrderCurrentSuryou` | Effective (remaining) quantity |
| `sOrderOrderPrice` | Order price |
| `sOrderCondition` | Execution condition |
| `sOrderOrderPriceKubun` | Price type: `""` = unused, `"1"` = market, `"2"` = limit, `"3"` = higher than parent, `"4"` = lower than parent |
| `sOrderGyakusasiOrderType` | Stop order type |
| `sOrderGyakusasiZyouken` | Stop trigger condition |
| `sOrderGyakusasiKubun` | Stop price type: `""` = unused, `"0"` = market, `"1"` = limit |
| `sOrderGyakusasiPrice` | Stop order price |
| `sOrderTriggerType` | Trigger type: `"0"` = untriggered, `"1"` = auto, `"2"` = manual order, `"3"` = manual expire |
| `sOrderTatebiType` | Position date type |
| `sOrderZougen` | Reverse increment (unused) |
| `sOrderYakuzyouSuryo` | Filled quantity |
| `sOrderYakuzyouPrice` | Fill price |
| `sOrderUtidekiKbn` | Partial fill flag: `""` = no split, `"2"` = split |
| `sOrderSikkouDay` | Execution date `YYYYMMDD` |
| `sOrderStatusCode` | Status code (see section 9.2) |
| `sOrderStatus` | Status name text |
| `sOrderYakuzyouStatus` | Fill status: `"0"` = unfilled, `"1"` = partial, `"2"` = full, `"3"` = filling |
| `sOrderOrderDateTime` | Order datetime `YYYYMMDDHHMMSS` |
| `sOrderOrderExpireDay` | Expiration date `YYYYMMDD` |
| `sOrderKurikosiOrderFlg` | Carryover flag: `"0"` = today, `"1"` = carryover, `"2"` = invalid |
| `sOrderCorrectCancelKahiFlg` | Correct/cancel allowed: `"0"` = both, `"1"` = neither, `"2"` = cancel only |
| `sGaisanDaikin` | Estimated trade amount |

### 4.6 CLMOrderListDetail (Order Detail Query)

**sCLMID**: `"CLMOrderListDetail"`
**Virtual URL**: `sUrlRequest`

#### Request Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `sCLMID` | string | Yes | `"CLMOrderListDetail"` |
| `sOrderNumber` | string | Yes | Order number to query |
| `sEigyouDay` | string | Yes | Business date `YYYYMMDD` |
| `p_no` | string | Yes | Request sequence number |
| `p_sd_date` | string | Yes | Client timestamp |

**Note**: All request parameters are required.

#### Response Fields

| Field | Type | Description |
|-------|------|-------------|
| `sCLMID` | string | `"CLMOrderListDetail"` |
| `sResultCode` | string | See CLMKabuNewOrder |
| `sResultText` | string | See CLMKabuNewOrder |
| `sWarningCode` | string | Warning code |
| `sWarningText` | string | Warning text |
| `sOrderNumber` | string | Order number |
| `sEigyouDay` | string | Business day |
| `sIssueCode` | string | Stock ticker |
| `sOrderSizyouC` | string | Market code |
| `sOrderBaibaiKubun` | string | Buy/Sell |
| `sGenkinSinyouKubun` | string | Cash/Margin type |
| `sOrderBensaiKubun` | string | Repayment type |
| `sOrderCondition` | string | Execution condition |
| `sOrderOrderPriceKubun` | string | Price type |
| `sOrderOrderPrice` | string | Order price |
| `sOrderOrderSuryou` | string | Order quantity |
| `sOrderCurrentSuryou` | string | Effective quantity |
| `sOrderStatusCode` | string | Status code (see section 9.2) |
| `sOrderStatus` | string | Status name |
| `sOrderOrderDateTime` | string | Order datetime `YYYYMMDDHHMMSS` |
| `sOrderOrderExpireDay` | string | Expiration date `YYYYMMDD` |
| `sChannel` | string | Channel: `"F"` = e-Shiten API, `"1"` = Standard Web, etc. |
| `sGenbutuZyoutoekiKazeiC` | string | Spot account type |
| `sSinyouZyoutoekiKazeiC` | string | Margin account type |
| `sGyakusasiOrderType` | string | Stop order type |
| `sGyakusasiZyouken` | string | Stop trigger condition |
| `sGyakusasiKubun` | string | Stop price type |
| `sGyakusasiPrice` | string | Stop price |
| `sTriggerType` | string | Trigger type |
| `sTriggerTime` | string | Trigger datetime `YYYYMMDDHHMMSS` |
| `sUkewatasiDay` | string | Settlement date `YYYYMMDD` |
| `sYakuzyouPrice` | string | Fill price |
| `sYakuzyouSuryou` | string | Fill quantity |
| `sBaiBaiDaikin` | string | Trade amount |
| `sUtidekiKubun` | string | Partial fill flag |
| `sGaisanDaikin` | string | Estimated trade amount |
| `sBaiBaiTesuryo` | string | Commission |
| `sShouhizei` | string | Consumption tax |
| `sTatebiType` | string | Position date type |
| `sSizyouErrorCode` | string | Exchange error code (`""` = OK; see CLMOrderErrReason master) |
| `sZougen` | string | Reverse increment (unused) |
| `sOrderAcceptTime` | string | Exchange accept/error time `YYYYMMDDHHMMSS` |
| `sOrderExpireDayLimit` | string | Order expiration date `YYYYMMDD` |
| `aYakuzyouSikkouList` | array | Fill/expiration detail list |
| `aKessaiOrderTategyokuList` | array | Settlement position list (margin) |

**aYakuzyouSikkouList array item fields:**

| Field | Description |
|-------|-------------|
| `sYakuzyouWarningCode` | Warning code |
| `sYakuzyouWarningText` | Warning text |
| `sYakuzyouSuryou` | Fill quantity |
| `sYakuzyouPrice` | Fill price |
| `sYakuzyouDate` | Fill datetime `YYYYMMDDHHMMSS` |

**aKessaiOrderTategyokuList array item fields:**

| Field | Description |
|-------|-------------|
| `sKessaiWarningCode` | Warning code |
| `sKessaiWarningText` | Warning text |
| `sKessaiTatebiZyuni` | Position priority |
| `sKessaiTategyokuDay` | Position date `YYYYMMDD` |
| `sKessaiTategyokuPrice` | Position price |
| `sKessaiOrderSuryo` | Repayment order quantity |
| `sKessaiYakuzyouSuryo` | Fill quantity |
| `sKessaiYakuzyouPrice` | Fill price |
| `sKessaiTateTesuryou` | Position commission |
| `sKessaiZyunHibu` | Daily interest |
| `sKessaiGyakuhibu` | Reverse daily interest |
| `sKessaiKakikaeryou` | Rewrite fee |
| `sKessaiKanrihi` | Management fee |
| `sKessaiKasikaburyou` | Stock lending fee |
| `sKessaiSonota` | Other fees |
| `sKessaiSoneki` | Settlement P&L / delivery amount |

---

## 5. Account / Position Queries

### 5.1 CLMGenbutuKabuList (Spot Stock Holdings List)

**sCLMID**: `"CLMGenbutuKabuList"`
**Virtual URL**: `sUrlRequest`

Retrieves the list of currently held spot (cash) stock positions.

#### Request Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `sCLMID` | string | Yes | `"CLMGenbutuKabuList"` |
| `sIssueCode` | string | No | Stock ticker (`""` = all holdings, e.g. `"7201"` = single stock) |
| `p_no` | string | Yes | Request sequence number |
| `p_sd_date` | string | Yes | Client timestamp |

**Note**: Summary totals (outside the list) are returned regardless of the ticker filter.

#### Response Fields (Outer)

| Field | Type | Description |
|-------|------|-------------|
| `sCLMID` | string | `"CLMGenbutuKabuList"` |
| `sResultCode` | string | See CLMKabuNewOrder |
| `sResultText` | string | See CLMKabuNewOrder |
| `sWarningCode` | string | Warning code |
| `sWarningText` | string | Warning text |
| `sIssueCode` | string | Echo of request value |
| `sIppanGaisanHyoukagakuGoukei` | string | Estimated valuation total (general account) |
| `sIppanGaisanHyoukaSonekiGoukei` | string | Estimated P&L total (general account) |
| `sNisaGaisanHyoukagakuGoukei` | string | Estimated valuation total (NISA account) |
| `sNisaGaisanHyoukaSonekiGoukei` | string | Estimated P&L total (NISA account) |
| `sNseityouGaisanHyoukagakuGoukei` | string | Estimated valuation total (NISA Growth account) |
| `sNseityouGaisanHyoukaSonekiGoukei` | string | Estimated P&L total (NISA Growth account) |
| `sTokuteiGaisanHyoukagakuGoukei` | string | Estimated valuation total (specific account) |
| `sTokuteiGaisanHyoukaSonekiGoukei` | string | Estimated P&L total (specific account) |
| `sTotalGaisanHyoukagakuGoukei` | string | Estimated valuation total (all accounts) |
| `sTotalGaisanHyoukaSonekiGoukei` | string | Estimated P&L total (all accounts) |
| `aGenbutuKabuList` | array | Holdings list; `""` if no data |

**aGenbutuKabuList array item fields:**

| Field | Description |
|-------|-------------|
| `sUriOrderWarningCode` | Warning code |
| `sUriOrderWarningText` | Warning text |
| `sUriOrderIssueCode` | Stock ticker |
| `sUriOrderZyoutoekiKazeiC` | Tax account type |
| `sUriOrderZanKabuSuryou` | Remaining shares |
| `sUriOrderUritukeKanouSuryou` | Sellable shares |
| `sUriOrderGaisanBokaTanka` | Estimated book value per share |
| `sUriOrderHyoukaTanka` | Valuation price per share |
| `sUriOrderGaisanHyoukagaku` | Valuation amount |
| `sUriOrderGaisanHyoukaSoneki` | Estimated P&L |
| `sUriOrderGaisanHyoukaSonekiRitu` | Estimated P&L ratio (%) |
| `sSyuzituOwarine` | Previous day closing price |
| `sZenzituHi` | Day-over-day change |
| `sZenzituHiPer` | Day-over-day change (%) |
| `sUpDownFlag` | Up/down flag: `"01"` = +5.01%+, `"06"` = unchanged, `"11"` = -5.01%- |
| `sNissyoukinKasikabuZan` | Securities finance lending balance |

### 5.2 CLMShinyouTategyokuList (Margin Position List)

**sCLMID**: `"CLMShinyouTategyokuList"`
**Virtual URL**: `sUrlRequest`

Retrieves margin (credit) position details.

#### Request Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `sCLMID` | string | Yes | `"CLMShinyouTategyokuList"` |
| `sIssueCode` | string | No | Stock ticker (`""` = all) |
| `p_no` | string | Yes | Request sequence number |
| `p_sd_date` | string | Yes | Client timestamp |

**Note**: Summary totals (outside the list) are returned regardless of the ticker filter.

#### Response Fields (Outer)

| Field | Type | Description |
|-------|------|-------------|
| `sCLMID` | string | `"CLMShinyouTategyokuList"` |
| `sResultCode` | string | See CLMKabuNewOrder |
| `sResultText` | string | See CLMKabuNewOrder |
| `sWarningCode` | string | Warning code |
| `sWarningText` | string | Warning text |
| `sIssueCode` | string | Echo of request value |
| `sUritateDaikin` | string | Total short position amount |
| `sKaitateDaikin` | string | Total long position amount |
| `sTotalDaikin` | string | Total position amount |
| `sHyoukaSonekiGoukeiUridate` | string | Estimated P&L total (short) |
| `sHyoukaSonekiGoukeiKaidate` | string | Estimated P&L total (long) |
| `sTotalHyoukaSonekiGoukei` | string | Total estimated P&L |
| `sTokuteiHyoukaSonekiGoukei` | string | P&L total (specific account) |
| `sIppanHyoukaSonekiGoukei` | string | P&L total (general account) |
| `aShinyouTategyokuList` | array | Position list; `""` if no data |

**aShinyouTategyokuList array item fields:**

| Field | Description |
|-------|-------------|
| `sOrderWarningCode` | Warning code |
| `sOrderWarningText` | Warning text |
| `sOrderTategyokuNumber` | Position number |
| `sOrderIssueCode` | Stock ticker |
| `sOrderSizyouC` | Market: `"00"` = TSE |
| `sOrderBaibaiKubun` | Buy/Sell |
| `sOrderBensaiKubun` | Repayment type: `"00"` = none, `"26"` / `"29"` / `"36"` / `"39"` |
| `sOrderZyoutoekiKazeiC` | Tax account: `"1"` = Specific, `"3"` = General, `"5"` = NISA, `"9"` = Corporate |
| `sOrderTategyokuSuryou` | Position shares |
| `sOrderTategyokuTanka` | Position unit price |
| `sOrderHyoukaTanka` | Valuation unit price |
| `sOrderGaisanHyoukaSoneki` | Estimated P&L |
| `sOrderGaisanHyoukaSonekiRitu` | Estimated P&L ratio (%) |
| `sTategyokuDaikin` | Position trade amount |
| `sOrderTateTesuryou` | Position commission |
| `sOrderZyunHibu` | Daily interest |
| `sOrderGyakuhibu` | Reverse daily interest |
| `sOrderKakikaeryou` | Rewrite fee |
| `sOrderKanrihi` | Management fee |
| `sOrderKasikaburyou` | Stock lending fee |
| `sOrderSonota` | Other fees |
| `sOrderTategyokuDay` | Position date `YYYYMMDD` |
| `sOrderTategyokuKizituDay` | Position expiry date (`"00000000"` = unlimited) |
| `sTategyokuSuryou` | Position quantity |
| `sOrderYakuzyouHensaiKabusu` | Filled repayment shares |
| `sOrderGenbikiGenwatasiKabusu` | Cash receipt/delivery shares |
| `sOrderOrderSuryou` | Pending order quantity |
| `sOrderHensaiKanouSuryou` | Repayable quantity |
| `sSyuzituOwarine` | Previous day closing price |
| `sZenzituHi` | Day-over-day change |
| `sZenzituHiPer` | Day-over-day change (%) |
| `sUpDownFlag` | Up/down flag |

**Important**: After cancelling a margin close-out order, wait and check `CLMShinyouTategyokuList` before placing a new close-out order. Immediate re-submission may cause "no margin position data" error.

### 5.3 CLMZanKaiKanougaku (Buying Power)

**sCLMID**: `"CLMZanKaiKanougaku"`
**Virtual URL**: `sUrlRequest`

Retrieves current buying power (available cash for trading).

#### Request Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `sCLMID` | string | Yes | `"CLMZanKaiKanougaku"` |
| `sIssueCode` | string | No | Unused (can be omitted) |
| `sSizyouC` | string | No | Unused (can be omitted) |
| `p_no` | string | Yes | Request sequence number |
| `p_sd_date` | string | Yes | Client timestamp |

**Note**: `sIssueCode` and `sSizyouC` are not required. Response fields are retained for compatibility.

#### Response Fields

| Field | Type | Description |
|-------|------|-------------|
| `sCLMID` | string | `"CLMZanKaiKanougaku"` |
| `sResultCode` | string | See CLMKabuNewOrder |
| `sResultText` | string | See CLMKabuNewOrder |
| `sWarningCode` | string | Warning code |
| `sWarningText` | string | Warning text |
| `sIssueCode` | string | Echo of request value |
| `sSizyouC` | string | Echo of request value |
| `sSummaryUpdate` | string | Last update datetime `YYYYMMDDHHMM` |
| `sSummaryGenkabuKaituke` | string | Spot stock buying power |
| `sSummaryNseityouTousiKanougaku` | string | NISA Growth investment available amount |
| `sHusokukinHasseiFlg` | string | Shortage flag: `"0"` = none, `"1"` = shortage occurred |

### 5.4 CLMZanShinkiKanoIjiritu (Margin Capacity & Maintenance Ratio)

**sCLMID**: `"CLMZanShinkiKanoIjiritu"`
**Virtual URL**: `sUrlRequest`

Returns margin capacity and maintenance ratio.

#### Request Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `sCLMID` | string | Yes | `"CLMZanShinkiKanoIjiritu"` |
| `sIssueCode` | string | No | Unused |
| `sSizyouC` | string | No | Unused |
| `p_no` | string | Yes | Request sequence number |
| `p_sd_date` | string | Yes | Client timestamp |

#### Response Fields

| Field | Type | Description |
|-------|------|-------------|
| `sCLMID` | string | `"CLMZanShinkiKanoIjiritu"` |
| `sResultCode` | string | See CLMKabuNewOrder |
| `sResultText` | string | See CLMKabuNewOrder |
| `sWarningCode` | string | Warning code |
| `sWarningText` | string | Warning text |
| `sIssueCode` | string | Echo of request value |
| `sSizyouC` | string | Echo of request value |
| `sSummaryUpdate` | string | Last update datetime `YYYYMMDDHHMM` |
| `sSummarySinyouSinkidate` | string | New margin position available amount |
| `sItakuhosyoukin` | string | Margin maintenance ratio (%) |
| `sOisyouKakuteiFlg` | string | Margin call flag: `"0"` = undetermined, `"1"` = confirmed |

### 5.5 CLMZanUriKanousuu (Sellable Quantity)

**sCLMID**: `"CLMZanUriKanousuu"`
**Virtual URL**: `sUrlRequest`

Returns the quantity of shares available for sale, broken down by account type.

#### Request Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `sCLMID` | string | Yes | `"CLMZanUriKanousuu"` |
| `sIssueCode` | string | Yes | Stock ticker (e.g., `"6501"`) |
| `p_no` | string | Yes | Request sequence number |
| `p_sd_date` | string | Yes | Client timestamp |

#### Response Fields

| Field | Type | Description |
|-------|------|-------------|
| `sCLMID` | string | `"CLMZanUriKanousuu"` |
| `sResultCode` | string | See CLMKabuNewOrder |
| `sResultText` | string | See CLMKabuNewOrder |
| `sWarningCode` | string | Warning code |
| `sWarningText` | string | Warning text |
| `sIssueCode` | string | Echo of request value |
| `sSummaryUpdate` | string | Last update datetime `YYYYMMDDHHMM` |
| `sZanKabuSuryouUriKanouIppan` | string | Sellable shares (general account) |
| `sZanKabuSuryouUriKanouTokutei` | string | Sellable shares (specific account) |
| `sZanKabuSuryouUriKanouNisa` | string | Sellable shares (NISA) |
| `sZanKabuSuryouUriKanouNseityou` | string | Sellable shares (NISA Growth) |

### 5.6 CLMZanKaiSummary (Account Summary)

**sCLMID**: `"CLMZanKaiSummary"`
**Virtual URL**: `sUrlRequest`

Returns a comprehensive summary of account status including buying power, margin status, order/fill counts, and deficit information.

**Important**: Do NOT poll this endpoint. Use EVENT I/F for fill notifications, then query account status only when triggered.

#### Request Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `sCLMID` | string | Yes | `"CLMZanKaiSummary"` |
| `p_no` | string | Yes | Request sequence number |
| `p_sd_date` | string | Yes | Client timestamp |

#### Response Fields

| Field | Type | Description |
|-------|------|-------------|
| `sCLMID` | string | `"CLMZanKaiSummary"` |
| `sResultCode` | string | See CLMKabuNewOrder |
| `sResultText` | string | See CLMKabuNewOrder |
| `sWarningCode` | string | Warning code |
| `sWarningText` | string | Warning text |
| `sUpdateDate` | string | Last update datetime `YYYYMMDDHHMM` |
| `sOisyouHasseiFlg` | string | Margin call flag: `"0"` = none, `"1"` = occurred |
| `sOhzsKeisanDay` | string | Margin call calculation date `YYYYMMDD` |
| `sOhzsGenkinHosyoukin` | string | Cash collateral |
| `sOhzsDaiyouHyoukagaku` | string | Substitute securities valuation |
| `sOhzsSasiireHosyoukin` | string | Deposited collateral |
| `sOhzsHyoukaSoneki` | string | P&L |
| `sOhzsSyokeihi` | string | Miscellaneous expenses |
| `sOhzsMiukeKessaiSon` | string | Unsettled settlement loss |
| `sOhzsMiukeKessaiEki` | string | Unsettled settlement profit |
| `sOhzsUkeireHosyoukin` | string | Received collateral |
| `sOhzsTatekabuDaikin` | string | Position trade amount |
| `sOhzsItakuHosyoukinRitu` | string | Margin maintenance ratio (%) |
| `sTatekaekinHasseiFlg` | string | Advance payment flag: `"0"` = none, `"1"` = occurred |
| `sThzNyukinKigenDay` | string | Payment deadline `YYYYMMDD` |
| `sThzSeisangaku` | string | Settlement amount |
| `sThzHibakariKousokukin` | string | Day trade restricted amount |
| `sThzHurikaegaku` | string | Transfer amount |
| `sThzHituyouNyukingaku` | string | Required deposit amount |
| `sThzKakuteiFlg` | string | Confirmed flag |
| `sGenbutuKabuKaituke` | string | Spot stock buying power |
| `sSinyouSinkidate` | string | New margin position available amount |
| `sSinyouGenbiki` | string | Cash receipt available amount |
| `sHosyouKinritu` | string | Margin maintenance ratio (%) |
| `sNseityouTousiKanougaku` | string | NISA Growth investment available |
| `sTousinKaituke` | string | Investment trust buying power |
| `sRuitouKaituke` | string | MMF/China Fund buying power |
| `sIPOKounyu` | string | IPO purchase available |
| `sSyukkin` | string | Withdrawable amount |
| `sFusokugaku` | string | Shortage amount |
| `sLargeKaidateYoryoku` | string | Futures long capacity |
| `sMiniKaidateYoryoku` | string | OP put sell capacity (mini) |
| `sLargeUridateYoryoku` | string | Futures short capacity |
| `sMiniUridateYoryoku` | string | OP call sell capacity (mini) |
| `sOpKaidateYoryokyu` | string | Options new long capacity |
| `sSyoukokinFusokugaku` | string | Margin shortage amount (today's claim) |
| `sGenbutuBaibaiDaikin` | string | Spot trade amount |
| `sGenbutuOrderCount` | string | Spot order count |
| `sGenbutuYakuzyouCount` | string | Spot fill count |
| `sSinyouBaibaiDaikin` | string | Margin trade amount |
| `sSinyouOrderCount` | string | Margin order count |
| `sSinyouYakuzyouCount` | string | Margin fill count |
| `sSakiBaibaiDaikin` | string | Futures trade amount |
| `sSakiOrderCount` | string | Futures order count |
| `sSakiYakuzyouCount` | string | Futures fill count |
| `sOpBaibaiDaikin` | string | Options trade amount |
| `sOpOrderCount` | string | Options order count |
| `sOpYakuzyouCount` | string | Options fill count |
| `aHikazeiKouzaList` | array | Tax-exempt account list |
| `aOisyouHasseiZyoukyouList` | array | Margin call occurrence list |
| `aHosyoukinSeikyuZyoukyouList` | array | Margin claim occurrence list |

**aHikazeiKouzaList array item fields:**

| Field | Description |
|-------|-------------|
| `sHikazeiTekiyouYear` | Applicable year `YYYY` |
| `sSeityouTousiKanougaku` | Growth investment available amount |

**aOisyouHasseiZyoukyouList array item fields:**

| Field | Description |
|-------|-------------|
| `sOhzHasseiDay` | Occurrence date `YYYYMMDD` |
| `sOhzHosyoukinRitu` | Margin ratio (%) |
| `sOhzNyukinKigenDay` | Payment deadline `YYYYMMDDHHMM` |
| `sOhzOisyouKingaku` | Margin call amount |
| `sOhzKakuteiFlg` | Confirmed flag |
| `sOhzHosyoukinZougen` | Margin change |
| `sOhzNyukin` | Deposit |
| `sOhzTategyokuKessai` | Position settlement |
| `sOhzKessaisonNyukin` | Settlement loss deposit |
| `sOhzMikaisyouKingaku` | Unresolved amount |
| `sOhzMikaisyouKingakuFlg` | Unresolved amount flag (unused) |

### 5.7 CLMZanKaiKanougakuSuii (Buying Power History)

**sCLMID**: `"CLMZanKaiKanougakuSuii"`
**Virtual URL**: `sUrlRequest`

Returns the history/trend of available trading amounts over 6 business days.

#### Request Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `sCLMID` | string | Yes | `"CLMZanKaiKanougakuSuii"` |
| `p_no` | string | Yes | Request sequence number |
| `p_sd_date` | string | Yes | Client timestamp |

#### Response Fields

| Field | Type | Description |
|-------|------|-------------|
| `sCLMID` | string | `"CLMZanKaiKanougakuSuii"` |
| `sResultCode` | string | See CLMKabuNewOrder |
| `sResultText` | string | See CLMKabuNewOrder |
| `sWarningCode` | string | Warning code |
| `sWarningText` | string | Warning text |
| `sUpdateDate` | string | Last update datetime `YYYYMMDDHHMM` |
| `sNearaiKubun` | string | Valuation status: `"0"` = stopped, `"1"` = in progress, `"2"` = complete |
| `aKanougakuSuiiList` | array | Available amount list (6 entries: [0]=today, [1]=2nd day, ... [5]=6th day) |

**aKanougakuSuiiList array item fields:**

| Field | Description |
|-------|-------------|
| `sHituke` | Date `YYYYMMDD` |
| `sAzukariKin` | Deposit balance |
| `sHattyuZyutoukin` | Outstanding order allocated amount |
| `sHibakariKousokukin` | Day trade restricted amount |
| `sSonotaKousokukin` | Other restricted amounts |
| `sGenkinHosyoukin` | Cash collateral |
| `sDaiyouHyoukagaku` | Substitute securities valuation |
| `sSasiireHosyoukin` | Deposited collateral |
| `sSinyouTateHyoukaSon` | Margin position valuation loss |
| `sSinyouTateHyoukaEki` | Margin position valuation profit |
| `sSinyouTadeSyoukeihi` | Margin position miscellaneous expenses |
| `sMiukewatasiKessaiSon` | Unsettled settlement loss |
| `sMiukewatasiKessaiEki` | Unsettled settlement profit |
| `sUkeireHosyoukin` | Received collateral |
| `sMikessaiTateDaikin` | Unsettled position amount |
| `sGenbikiWatasiTateDaikin` | Cash receipt/delivery position amount |
| `sHituyouHosyoukin` | Required collateral |
| `sHituyouGenkinHosyoukin` | Required cash collateral |
| `sHosyoukinYoryoku` | Collateral margin |
| `sGenkinHosyoukinYoryoku` | Cash collateral margin |
| `sItakuHosyoukinRitu` | Margin maintenance ratio (%) |
| `sHosyoukinHikidasiKousokukin` | Collateral withdrawal restriction |
| `sHosyoukinHikidasiYoryoku` | Collateral withdrawal margin |
| `sOisyouHituyouHosyoukin` | Margin call required collateral |
| `sOisyouYoryoku` | Margin call margin |
| `sFusokugaku` | Shortage amount |
| `sGenbutuKaitukeKanougaku` | Spot stock buying power |
| `sSinyouSinkidateKanougaku` | New margin position available |
| `sGenbikiKanougaku` | Cash receipt available |
| `sTousinKaitukeKanougaku` | Investment trust buying power |
| `sSyukkinKanougaku` | Withdrawable amount |

### 5.8 CLMZanKaiGenbutuKaitukeSyousai (Spot Stock Purchase Available Amount Detail)

**sCLMID**: `"CLMZanKaiGenbutuKaitukeSyousai"`
**Virtual URL**: `sUrlRequest`

Detailed breakdown of available amount for spot stock purchases.

#### Request Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `sCLMID` | string | Yes | `"CLMZanKaiGenbutuKaitukeSyousai"` |
| `sHitukeIndex` | string | Yes | Date index: `"3"` = 4th business day, `"4"` = 5th, `"5"` = 6th |
| `p_no` | string | Yes | Request sequence number |
| `p_sd_date` | string | Yes | Client timestamp |

#### Response Fields

| Field | Type | Description |
|-------|------|-------------|
| `sCLMID` | string | `"CLMZanKaiGenbutuKaitukeSyousai"` |
| `sResultCode` | string | See CLMKabuNewOrder |
| `sResultText` | string | See CLMKabuNewOrder |
| `sWarningCode` | string | Warning code |
| `sWarningText` | string | Warning text |
| `sHitukeIndex` | string | Echo of request value |
| `sHituke` | string | Specified date `YYYYMMDD` |
| `sGenkinHosyoukin` | string | Cash collateral |
| `sHosyoukinGenbutuKaitukeKanouga` | string | Spot buying power from collateral |
| `sGenbutuKaitukeKanougaku` | string | Spot stock buying power |
| `sAzukariKin` | string | Deposit balance |
| `sHattyuZyutoukin` | string | Outstanding order allocated amount |
| `sHibakariKousokukin` | string | Day trade restricted amount |
| `sSonotaKousokukin` | string | Other restricted amounts |
| `sHituyouGenkinHosyoukin` | string | Required cash collateral |
| `sDaiyouHyoukagaku` | string | Substitute securities valuation |
| `sTatekabuHyoukaSoneki` | string | Position P&L |
| `sTatekabuSyoukeihi` | string | Position miscellaneous expenses |
| `sMiukewatasiKessaiSon` | string | Unsettled settlement loss |
| `sMiukewatasiKessaiEki` | string | Unsettled settlement profit |
| `sUkeireHosyoukin` | string | Received collateral |
| `sHituyouHosyoukin` | string | Required collateral |
| `sHosyoukinYoryoku` | string | Collateral margin |

### 5.9 CLMZanKaiSinyouSinkidateSyousai (Margin New Position Available Amount Detail)

**sCLMID**: `"CLMZanKaiSinyouSinkidateSyousai"`
**Virtual URL**: `sUrlRequest`

Detailed breakdown of available amount for new margin positions.

#### Request Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `sCLMID` | string | Yes | `"CLMZanKaiSinyouSinkidateSyousai"` |
| `sHitukeIndex` | string | Yes | Date index: `"0"`-`"5"` (1st through 6th business day) |
| `p_no` | string | Yes | Request sequence number |
| `p_sd_date` | string | Yes | Client timestamp |

#### Response Fields

| Field | Type | Description |
|-------|------|-------------|
| `sCLMID` | string | `"CLMZanKaiSinyouSinkidateSyousai"` |
| `sResultCode` | string | See CLMKabuNewOrder |
| `sResultText` | string | See CLMKabuNewOrder |
| `sWarningCode` | string | Warning code |
| `sWarningText` | string | Warning text |
| `sHitukeIndex` | string | Echo of request value |
| `sHituke` | string | Specified date `YYYYMMDD` |
| `sUkeireHosyoukin` | string | Received collateral |
| `sHituyouHosyoukin` | string | Required collateral |
| `sHosyoukinYoryoku` | string | Collateral margin |
| `sHosyoukinTyousyuRitu` | string | Collateral collection ratio (%) |
| `sSinyouSinkidateKanougaku` | string | New margin position available |
| `sAzukariKin` | string | Deposit balance |
| `sHattyuZyutoukin` | string | Outstanding order allocated amount |
| `sSonotaKousokukin` | string | Other restricted amounts |
| `sGenkinHosyoukin` | string | Cash collateral |
| `sDaiyouHyoukagaku` | string | Substitute securities valuation |
| `sHattyuDaiyouHyoukagaku` | string | Spot buy order substitute securities valuation |
| `sSasiireHosyoukin` | string | Deposited collateral |
| `sSinkiTesuryou` | string | New position commission |
| `sHibuGyakuhibuKousokukin` | string | Daily/reverse interest restricted amount |
| `sHibuGyakuhibuSyueki` | string | Daily/reverse interest income |
| `sSonotaTateSyokeihi` | string | Other uncollected expenses |
| `sSinyouTadeSyoukeihi` | string | Position miscellaneous expenses |
| `sSinyouTateHyoukaSon` | string | Position valuation loss |
| `sSinyouTateHyoukaEki` | string | Position valuation profit |
| `sTatekabuHyoukaSoneki` | string | Position P&L |
| `sMiukewatasiKessaiSon` | string | Unsettled settlement loss |
| `sMiukewatasiKessaiEki` | string | Unsettled settlement profit |
| `sSaiteiHituyouHosyoukin` | string | Minimum required collateral |
| `sHosyoukin` | string | Position required collateral |
| `sHattyuHosyoukin` | string | Order required collateral |
| `sGenbikiWatasiHosyoukin` | string | Cash receipt/delivery required collateral |
| `sMikessaiGenkinHosyoukin` | string | Position required cash collateral |
| `sHattyuGenkinHosyoukin` | string | Order required cash collateral |
| `sGenbwGenkinHosyoukin` | string | Cash receipt/delivery required cash collateral |
| `sHituyouGenkinHosyoukin` | string | Required cash collateral |
| `sHosyoukinRitu` | string | Collateral ratio (%) |
| `sHosyoukinIziRitu` | string | Collateral maintenance ratio (%) |
| `sHosyoukinRituIziYoryoku` | string | Collateral ratio maintenance margin |
| `sHosyoukinIzirituIziYoryoku` | string | Collateral maintenance ratio margin |
| `sMikessaiTateDaikin` | string | Unsettled position amount |
| `sHattyuTateDaikin` | string | Order position amount |
| `sGenbikiWatasiTateDaikin` | string | Cash receipt/delivery position amount |
| `sItakuHosyoukinRitu` | string | Margin maintenance ratio (%) |
| `sTouzituKessaiSon` | string | Today's settlement loss |
| `sTouzituKessaiEki` | string | Today's settlement profit |
| `sKessaiTotalToday` | string | Today's settlement P&L total |
| `sTouzituKessaiZenHyouka` | string | Today's settlement previous day valuation |
| `sUkewatasiTategyokuSon` | string | Specified day settlement loss |
| `sUkewatasiTategyokuEki` | string | Specified day settlement profit |
| `sKessaiTotalSiteibi` | string | Specified day settlement P&L cumulative |

### 5.10 CLMZanRealHosyoukinRitu (Real-time Margin Ratio)

**sCLMID**: `"CLMZanRealHosyoukinRitu"`
**Virtual URL**: `sUrlRequest`

Real-time margin guarantee rate with comparison values (T+0 and T+5 perspectives).

#### Request Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `sCLMID` | string | Yes | `"CLMZanRealHosyoukinRitu"` |
| `p_no` | string | Yes | Request sequence number |
| `p_sd_date` | string | Yes | Client timestamp |

#### Response Fields

| Field | Type | Description |
|-------|------|-------------|
| `sCLMID` | string | `"CLMZanRealHosyoukinRitu"` |
| `sResultCode` | string | See CLMKabuNewOrder |
| `sResultText` | string | See CLMKabuNewOrder |
| `sWarningCode` | string | Warning code |
| `sWarningText` | string | Warning text |
| `sSasiireHosyoukin` | string | Deposited collateral |
| `sHyoukaSonEki` | string | P&L |
| `sUkeireHosyoukin` | string | Received collateral |
| `sTateKabuDaikin` | string | Position amount |
| `sItakuHosyoukinRitu` | string | Margin maintenance ratio (%) |
| `sOisyouHituyouHosyoukin` | string | Margin call required collateral |
| `sOisyouYoryoku` | string | Margin call margin |
| `sT0SasiireHosyoukin` | string | [T+0] Deposited collateral |
| `sT0HyoukaSonEki` | string | [T+0] P&L |
| `sT0UkeireHosyoukin` | string | [T+0] Received collateral |
| `sT0TateKabuDaikin` | string | [T+0] Position amount |
| `sT0ItakuHosyoukinRitu` | string | [T+0] Margin ratio (%) |
| `sT0OisyouHituyouHosyoukin` | string | [T+0] Margin call required collateral |
| `sT0OisyouYoryoku` | string | [T+0] Margin call margin |
| `sT5SasiireHosyoukin` | string | [T+5] Deposited collateral |
| `sT5HyoukaSonEki` | string | [T+5] P&L |
| `sT5UkeireHosyoukin` | string | [T+5] Received collateral |
| `sT5TateKabuDaikin` | string | [T+5] Position amount |
| `sT5ItakuHosyoukinRitu` | string | [T+5] Margin ratio (%) |
| `sT5OisyouHituyouHosyoukin` | string | [T+5] Margin call required collateral |
| `sT5OisyouYoryoku` | string | [T+5] Margin call margin |

---

## 6. Master Data (REQUEST I/F)

Master data is accessed via `sUrlMaster` virtual URL.

### 6.1 CLMEventDownload (Master Data Download)

**sCLMID**: `"CLMEventDownload"`

A streaming download mechanism (not standard request-response). After the initial request, the server continuously sends records until download is complete, then sends update records as they occur during the trading day.

#### Request Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `sCLMID` | string | Yes | `"CLMEventDownload"` |
| `sTargetCLMID` | string | No | Comma-separated list of target master data IDs. Empty = all masters. |

**Available sTargetCLMID values:**
- `CLMSystemStatus` - System status
- `CLMDateZyouhou` - Date information
- `CLMYobine` - Tick size (price increments)
- `CLMUnyouStatus` - Operation status by state
- `CLMUnyouStatusKabu` - Stock market operation status
- `CLMUnyouStatusHasei` - Derivative market operation status
- `CLMIssueMstKabu` - Stock issue master
- `CLMIssueSizyouMstKabu` - Stock issue-market master
- `CLMIssueSizyouKiseiKabu` - Stock issue-market regulations
- `CLMIssueMstSak` - Futures issue master
- `CLMIssueMstOp` - Options issue master
- `CLMIssueSizyouKiseiHasei` - Derivative issue-market regulations
- `CLMDaiyouKakeme` - Substitute collateral rate
- `CLMHosyoukinMst` - Margin requirement master
- `CLMOrderErrReason` - Exchange error/reason codes
- `CLMEventDownloadComplete` - Download complete marker

**Characteristics**:
- Takes approximately 40 seconds to complete initial download
- Can be run in parallel with other virtual URLs
- Multiple master downloads can run in parallel with each other
- Continues streaming until client disconnects or logs out
- This is a streaming request; no response is returned. Data is delivered until `CLMEventDownloadComplete` is received.

**Example**: To download only stock master, specify `sTargetCLMID = "CLMIssueMstKabu,CLMEventDownloadComplete"` and disconnect after receiving `CLMEventDownloadComplete`.

#### Master Data Notes (v4r4+)

- Excluded from stock master download: OTC stocks (market code `09`)
- Excluded from futures/options: expired contracts (delivery month < business month)
- TSE-only for certain fields: substitute securities valuation price (stock master), price range limits/previous close (market master) -- non-TSE markets return empty strings
- For fields without explicit value definitions, empty string or `"0"` means "no value"

### 6.2 CLMMfdsGetMasterData (Master Data Query)

**sCLMID**: `"CLMMfdsGetMasterData"` (v4r2+)
**Virtual URL**: `sUrlMaster`

A request-response API that allows querying specific master data items. More efficient than full download when only specific data is needed.

#### Request Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `sCLMID` | string | Yes | `"CLMMfdsGetMasterData"` |
| `sTargetCLMID` | string | Yes | Comma-separated list of target master IDs |
| `sTargetColumn` | string | No | Comma-separated list of fields to return (`""` = all fields) |
| `p_no` | string | Yes | Request sequence number |
| `p_sd_date` | string | Yes | Client timestamp |

**Available sTargetCLMID values:**
- `CLMIssueMstKabu` - Stock issue master
- `CLMIssueSizyouMstKabu` - Stock issue-market master
- `CLMIssueMstSak` - Futures issue master
- `CLMIssueMstOp` - Options issue master
- `CLMIssueMstOther` - Other (index + FX) issue master (unique to API)
- `CLMIssueMstIndex` - Index issue master (separate from Other)
- `CLMIssueMstFx` - FX issue master (separate from Other)
- `CLMOrderErrReason` - Exchange error codes
- `CLMDateZyouhou` - Date information

**Note**: `CLMIssueMstOther` returns both index and FX data. `CLMIssueMstIndex` and `CLMIssueMstFx` allow fetching them individually.

#### Response

Returns arrays of matching master data records with the requested fields. Field definitions follow the same structure as `CLMEventDownload` master data.

### 6.3 CLMMfdsGetNewsHead (News Header Query)

**sCLMID**: `"CLMMfdsGetNewsHead"` (v4r4+)
**Virtual URL**: `sUrlMaster`

Retrieves news headline/header information.

#### Request Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `sCLMID` | string | Yes | `"CLMMfdsGetNewsHead"` |
| `p_CG` | string | No | Category code (single) |
| `p_IS` | string | No | Stock ticker code (single) |
| `p_DT_FROM` | string | No | Date range start `YYYYMMDD` |
| `p_DT_TO` | string | No | Date range end `YYYYMMDD` |
| `p_REC_OFST` | string | No | Record offset (default: 0 = most recent) |
| `p_REC_LIMT` | string | No | Max record count (default: 100, max: 100) |
| `p_no` | string | Yes | Request sequence number |
| `p_sd_date` | string | Yes | Client timestamp |

**Note**: All parameters except sCLMID are optional AND-condition filters. Results are sorted by news ID descending (newest first).

#### Response Fields

| Field | Type | Description |
|-------|------|-------------|
| `sCLMID` | string | `"CLMMfdsGetNewsHead"` |
| `p_REC_MAX` | string | Total matching record count (not array size) |
| `aCLMMfdsNewsHead` | array | News header list |

**aCLMMfdsNewsHead array item fields:**

| Field | Description |
|-------|-------------|
| `p_ID` | News ID (unique identifier) |
| `p_DT` | News date `YYYYMMDD` |
| `p_TM` | News time `HHMMSS` |
| `p_CGL` | Category list (pipe-delimited) |
| `p_GNL` | Genre list (pipe-delimited) |
| `p_ISL` | Related stock code list (pipe-delimited) |
| `p_HDL` | Headline (Shift-JIS BASE64 encoded) |

### 6.4 CLMMfdsGetNewsBody (News Body Query)

**sCLMID**: `"CLMMfdsGetNewsBody"` (v4r4+)
**Virtual URL**: `sUrlMaster`

Retrieves full news article body text.

#### Request Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `sCLMID` | string | Yes | `"CLMMfdsGetNewsBody"` |
| `p_ID` | string | Yes | News ID (from CLMMfdsGetNewsHead) |
| `p_no` | string | Yes | Request sequence number |
| `p_sd_date` | string | Yes | Client timestamp |

#### Response Fields

| Field | Type | Description |
|-------|------|-------------|
| `sCLMID` | string | `"CLMMfdsGetNewsBody"` |
| `p_ID` | string | Echo of request value |
| `aCLMMfdsNewsBody` | array | News body list |

**aCLMMfdsNewsBody array item fields:**

| Field | Description |
|-------|-------------|
| `p_ID` | News ID |
| `p_DT` | News date `YYYYMMDD` |
| `p_TM` | News time `HHMMSS` |
| `p_CGL` | Category list (pipe-delimited) |
| `p_GNL` | Genre list (pipe-delimited) |
| `p_ISL` | Related stock code list (pipe-delimited) |
| `p_HDL` | Headline (Shift-JIS BASE64 encoded) |
| `p_TX` | Body text (Shift-JIS BASE64 encoded) |

### 6.5 CLMMfdsGetIssueDetail (Issue Detail Query)

**sCLMID**: `"CLMMfdsGetIssueDetail"` (v4r6+)
**Virtual URL**: `sUrlMaster`

Retrieves detailed fundamental information about specific stocks.

#### Request Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `sCLMID` | string | Yes | `"CLMMfdsGetIssueDetail"` |
| `sTargetIssueCode` | string | Yes | Comma-separated stock codes (max 120) |
| `p_no` | string | Yes | Request sequence number |
| `p_sd_date` | string | Yes | Client timestamp |

#### Response Fields

| Field | Type | Description |
|-------|------|-------------|
| `sCLMID` | string | `"CLMMfdsGetIssueDetail"` |
| `aCLMMfdsIssueDetail` | array | Detail list |

**aCLMMfdsIssueDetail array item fields:**

| Field | Description |
|-------|-------------|
| `sIssueCode` | Stock ticker |
| `pBPSB` | BPS (actual) / per-share assets |
| `pCLOE` | Ex-dividend date (full year) `YYYY/MM/DD` |
| `pEPSF` | EPS (forecast) / per-share earnings |
| `pEXRD` | Last ex-date (non-fiscal) `YYYY/MM/DD` |
| `pIDVE` | Ex-dividend date (interim) `YYYY/MM/DD` |
| `pROEL` | ROE (forecast) |
| `pRPER` | PER (forecast, consolidated priority) |
| `pSPBR` | PBR (actual, simple) |
| `pSPRO` | Earnings yield (forecast, simple) |
| `pSYIE` | Dividend yield (forecast, simple) |
| `pYHPD` | YTD high date `YYYY/MM/DD` |
| `pYHPR` | YTD high price |
| `pYLPD` | YTD low date `YYYY/MM/DD` |
| `pYLPR` | YTD low price |

### 6.6 CLMMfdsGetSyoukinZan (Securities Finance Balance Query)

**sCLMID**: `"CLMMfdsGetSyoukinZan"` (v4r6+)
**Virtual URL**: `sUrlMaster`

Retrieves securities finance (Nisshokin) balance information.

#### Request Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `sCLMID` | string | Yes | `"CLMMfdsGetSyoukinZan"` |
| `sTargetIssueCode` | string | Yes | Comma-separated stock codes (max 120) |
| `p_no` | string | Yes | Request sequence number |
| `p_sd_date` | string | Yes | Client timestamp |

#### Response Fields

| Field | Type | Description |
|-------|------|-------------|
| `sCLMID` | string | `"CLMMfdsGetSyoukinZan"` |
| `aCLMMfdsSyoukinZan` | array | Balance list |

**aCLMMfdsSyoukinZan array item fields:**

| Field | Description |
|-------|-------------|
| `sIssueCode` | Stock ticker |
| `pSFC6` | Net balance day-over-day change |
| `pSFD` | Update date `YYYY/MM/DD` |
| `pSFD6` | Turnover days |
| `pSFF6` | Lending balance |
| `pSFG6` | Lending day-over-day change |
| `pSFKS` | Status: `"1"` = preliminary, `"2"` = final |
| `pSFL6` | Lending new |
| `pSFN6` | (additional lending fields) |

### 6.7 CLMMfdsGetShinyouZan (Margin Balance Query)

**sCLMID**: `"CLMMfdsGetShinyouZan"` (v4r6+)
**Virtual URL**: `sUrlMaster`

Retrieves credit/margin trading balance information.

#### Request Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `sCLMID` | string | Yes | `"CLMMfdsGetShinyouZan"` |
| `sTargetIssueCode` | string | Yes | Comma-separated stock codes (max 120) |
| `p_no` | string | Yes | Request sequence number |
| `p_sd_date` | string | Yes | Client timestamp |

### 6.8 CLMMfdsGetHibuInfo (Reverse Daily Interest Query)

**sCLMID**: `"CLMMfdsGetHibuInfo"` (v4r6+)
**Virtual URL**: `sUrlMaster`

Retrieves reverse daily interest (premium charge for short selling) information.

#### Request Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `sCLMID` | string | Yes | `"CLMMfdsGetHibuInfo"` |
| `sTargetIssueCode` | string | Yes | Comma-separated stock codes (max 120) |
| `p_no` | string | Yes | Request sequence number |
| `p_sd_date` | string | Yes | Client timestamp |

#### Response Fields

| Field | Type | Description |
|-------|------|-------------|
| `sCLMID` | string | `"CLMMfdsGetHibuInfo"` |
| `aCLMMfdsHibuInfo` | array | Reverse daily interest list |

**aCLMMfdsHibuInfo array item fields:**

| Field | Description |
|-------|-------------|
| `sIssueCode` | Stock ticker |
| `pBWRQ` | Reverse daily interest value |

### 6.9 CLMMfdsGetMarketPrice (Market Price Query)

**sCLMID**: `"CLMMfdsGetMarketPrice"` (v4r2+)
**Virtual URL**: `sUrlPrice`

Retrieves current market prices for specified stocks. Supports up to **120 stock codes** per request.

#### Request Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `sCLMID` | string | Yes | `"CLMMfdsGetMarketPrice"` |
| `sTargetIssueCode` | string | Yes | Comma-separated stock codes (max 120) |
| `sTargetColumn` | string | No | Comma-separated information codes to retrieve |
| `p_no` | string | Yes | Request sequence number |
| `p_sd_date` | string | Yes | Client timestamp |

**Note**: Information codes use the type+code format specified in the EVENT I/F data specification document section 3.(3) FD.

#### Response Fields

| Field | Type | Description |
|-------|------|-------------|
| `sCLMID` | string | `"CLMMfdsGetMarketPrice"` |
| `aCLMMfdsMarketPrice` | array | Price list |

**aCLMMfdsMarketPrice array item fields:**

| Field | Description |
|-------|-------------|
| `sIssueCode` | Stock ticker |
| (various `p*` fields) | Price data fields matching requested information codes (e.g., `pDPP` = closing price, `pPRP` = previous close) |
| (various `t*:T` fields) | Timestamp fields (e.g., `tDPP:T` = closing price timestamp) |

### 6.10 CLMMfdsGetMarketPriceHistory (Historical Price Query)

**sCLMID**: `"CLMMfdsGetMarketPriceHistory"` (v4r3+)
**Virtual URL**: `sUrlPrice`

Retrieves historical OHLCV data for a specified stock. Data goes back approximately 20 years. Returns data in date ascending order.

#### Request Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `sCLMID` | string | Yes | `"CLMMfdsGetMarketPriceHistory"` |
| `sIssueCode` | string | Yes | Single stock ticker (e.g., `"6501"`) |
| `sSizyouC` | string | No | Market code (default: `"00"` = TSE) |
| `p_no` | string | Yes | Request sequence number |
| `p_sd_date` | string | Yes | Client timestamp |

#### Response Fields

| Field | Type | Description |
|-------|------|-------------|
| `sCLMID` | string | `"CLMMfdsGetMarketPriceHistory"` |
| `sIssueCode` | string | Echo of request value |
| `sSizyouC` | string | Echo of request value |
| `aCLMMfdsGetMarketPriceHistory` | array | Historical data list |

**aCLMMfdsGetMarketPriceHistory array item fields:**

| Field | Description |
|-------|-------------|
| `sDate` | Date `YYYYMMDD` |
| `pDOP` | Open price |
| `pDHP` | High price |
| `pDLP` | Low price |
| `pDPP` | Close price |
| `pDV` | Volume |
| `pDOPxK` | Open price x split factor |
| `pDHPxK` | High price x split factor |
| `pDLPxK` | Low price x split factor |
| `pDPPxK` | Close price x split factor |
| `pDVxK` | Volume / split factor |
| `pSPUO` | Pre-split unit count (only on split dates) |
| `pSPUC` | Post-split unit count (only on split dates) |
| `pSPUK` | Split conversion factor = pre/post (only on split dates) |

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
| `FD` | Tick Data | Real-time tick/quote data |
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
| `sOrderStatusCode` | string | Order status code (see section 9.2) |
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

Real-time market prices via EVENT I/F are subject to throttling. Prices may be delayed depending on the client's network conditions. This is a best-effort delivery mechanism.

### 7.8 WebSocket Service Note

If server load becomes problematic due to WebSocket connections, Tachibana may immediately suspend the WebSocket service. HTTP Chunk EVENT I/F will continue to function. Applications should be designed to fall back to HTTP Chunk if WebSocket becomes unavailable.

---

## 8. EVENT I/F (WebSocket) — Detailed Specification

> **Important**: EVENT I/F uses a proprietary format, NOT JSON. The REQUEST I/F compress/uncompress mechanism does NOT apply here. See `api_event_if_v4r7.pdf` for the full specification.

### 8.1 Connection Method

The WebSocket URL is obtained from the login response field `sUrlEventWebSocket`. Connect using WebSocket protocol with query parameters:

```
wss://...?p_rid=0&p_board_no=1000&p_eno=0&p_evt_cmd=EC
```

- **NOT JSON** — uses a proprietary binary/text format
- ShiftJIS text items are **BASE64 encoded** in the WebSocket version

### 8.2 Query Parameters

| Param | Description | Example |
|-------|-------------|---------|
| `p_rid` | Board application ID (0 for API without price feed) | `0` |
| `p_board_no` | Board number (1000 for API) | `1000` |
| `p_gyou_no` | Row number (optional) | - |
| `p_issue_code` | Issue code (optional) | - |
| `p_mkt_code` | Market code (optional) | - |
| `p_eno` | Event number to resume from (0 = all) | `0` |
| `p_evt_cmd` | Event types to subscribe, comma-separated | `ST,KP,EC` |

### 8.3 Data Format

- Field separator: `^A` (0x01)
- Key/value separator: `^B` (0x02)
- Value-internal separator: `^C` (0x03)

Example:
```
p_no^B208^Ap_date^B2018.12.03^Ap_cmd^BST
```

### 8.4 Common Notification Fields

| No | Field | Description |
|----|-------|-------------|
| 1 | `p_no` | Sequence number (per connection, starting from 1) |
| 2 | `p_date` | Notification timestamp |
| 3 | `p_cmd` | Event type (`ST`, `KP`, `FD`, `EC`, `NS`, etc.) |

### 8.5 Event Types (p_evt_cmd)

| No | Value | Description | Notes |
|----|-------|-------------|-------|
| 1 | `ST` | Error status | Sent on error, then disconnects |
| 2 | `KP` | Keep-alive | Every 5 seconds if no other notifications |
| 3 | `FD` | Market price data | Full snapshot first, then changes only |
| 4 | `EC` | Order/execution events | All undeleted events for the day, then real-time |
| 5 | `NS` | News | All undeleted news for the day, then real-time |
| 7 | `SS` | System status | Same as above |
| 8 | `US` | Operation status | Same as above |

### 8.6 EC (Order/Execution) Notification Fields

| No | Field | Example | Description |
|----|-------|---------|-------------|
| 1 | `p_PV` | `MSGSV` | Provider |
| 2 | `p_ENO` | `10507` | Event number (unique per business day) |
| 3 | `p_ALT` | `1` | Alert flag (`1`=initial, `0`=resend) |
| 4 | `p_NT` | `100` | Notification type (see below) |
| 5 | `p_ON` | `3000945` | Order number |
| 6 | `p_ED` | `20181203` | Business date (YYYYMMDD) |
| 7 | `p_OON` | `0` | Parent order number (`0`=parent) |
| 8 | `p_OT` | `1` | Order type (`1`=parent, `2`=child) |
| 9 | `p_ST` | `1` | Product type (`1`=stock, `3`=futures, `4`=options) |
| 10 | `p_IC` | `2468` | Issue code |
| 11 | `p_MC` | `00` | Market code |
| 12 | `p_BBKB` | `1` | Buy/Sell (`1`=sell, `3`=buy) |
| 13 | `p_THKB` | `0` | Trade type (`0`=spot, `2`=margin new, `4`=margin close) |
| 14 | `p_CRSJ` | `0` | Execution condition (`0`=none, `2`=opening, `4`=closing, `6`=fail) |
| 15 | `p_CRPRKB` | `2` | Price type (`0`=unused, `1`=market, `2`=limit) |
| 19 | `p_CRPR` | `850.000000` | Order price |
| 20 | `p_CRSR` | `5300` | Order quantity |
| 21 | `p_CRTKSR` | `0` | Cancelled quantity |
| 22 | `p_CREPSR` | `0` | Expired quantity |
| 23 | `p_CREXSR` | `0` | Executed quantity |
| 24 | `p_ODST` | `0` | Order status (`0`=pending, `1`=accepted, `2`=error, `3`=partial expire, `4`=all expire, `5`=carried) |
| 26 | `p_TTST` | `0` | Correction/Cancel status (`0`=none, `1`-`9` various states) |
| 27 | `p_EXST` | `0` | Execution status (`0`=unfilled, `1`=partial, `2`=filled, `3`=filling) |
| 28 | `p_LMIT` | `20181211` | Expiry date (YYYYMMDD) |
| 31 | `p_EPRC` | `""` | Expiry reason code (from exchange) |
| 32 | `p_EXPR` | `0.000000` | Execution price (from exchange) |
| 33 | `p_EXSR` | `0` | Execution quantity (from exchange) |
| 34 | `p_EXRC` | `""` | Exchange error code |
| 35 | `p_EXDT` | `20181203132243` | Notification datetime (YYYYMMDDHHMMSS) |
| 36 | `p_IN` | `フュートレック` | Issue name (BASE64 in WebSocket) |

#### p_NT (Notification Type) Values

| p_NT | Description |
|------|-------------|
| `1` | 注文受付 (Order accepted) |
| `2` | 訂正受付 (Correction accepted) |
| `3` | 取消受付 (Cancel accepted) |
| `4`-`6` | 受付エラー (Acceptance errors) |
| `7`-`9` | 登録エラー (Registration errors) |
| `10` | 訂正完了 (Correction complete) |
| `11` | 取消完了 (Cancel complete) |
| `12` | 約定成立 (Execution filled) |
| `13` | 失効 (Expired) |
| `14` | 失効（連続注文） (Expired — chained order) |
| `100` | 注文状態変更 (Status change) |

### 8.7 ST (Error Status) Fields

| No | Field | Description |
|----|-------|-------------|
| 1 | `p_errno` | Error code (see p_errno table below) |
| 2 | `p_err` | Error message |

### 8.8 p_errno Error Codes (EVENT I/F)

| p_errno | p_err | Description |
|---------|-------|-------------|
| `0` | `""` | No problem |
| `1` | `""` | Board no data |
| `2` | `セッションが切断しました。` | Session inactive |
| `9` | `システム、サービス停止中。` | Service offline |
| `-1` | `引数エラー。` | Parameter error |
| `-2` | *(message)* | Database access error |
| `-3` | *(message)* | SAPSV access error |
| `-12` | `システム、サービス停止中。` | Service is offline |
| `-62` | `システム、情報提供時間外。` | Stockhouse is offline |

---

## 9. Error Handling

### 9.1 p_errno (Request-Level Errors)

| p_errno | Description |
|---------|-------------|
| `0` | Success |
| `-2` | Parameter error |
| `2` | Virtual URL invalid (session expired, logged out, or re-authenticated) |
| `8` | Time synchronization error (`p_sd_date` >30s behind server time) |
| Other non-zero | System error (e-Shiten system down, closed, or API subsystem failure) |

When `p_errno != 0`, the `p_err` field contains the error message text.

### 9.2 sOrderStatusCode (Order Status Codes)

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

### 9.3 sResultCode (Business Result Codes)

| sResultCode | Description |
|-------------|-------------|
| `"0"` | Success |
| Non-zero | Business error (details in `sResultText`) |

### 9.4 Warning Codes

Phone verification related warnings were added in v4r7. Refer to the manual section 7 (result/warning code table) for the complete list.

---

## 10. Important Notes

### 10.1 Rate Limits

- **10 requests/second** per customer, enforced server-side
- Excessive load may result in account suspension

### 10.2 Operating Hours

- Available during e-Shiten system operating hours
- System closure at approximately 03:30 JST daily
- Maintenance and holidays follow e-Shiten standard web schedule
- If the API subsystem fails but e-Shiten is operational, use the standard web interface for order management

### 10.3 Encoding

- **Request**: JSON with Shift-JIS encoding for Japanese characters. When using GET, the JSON is URL-encoded.
- **Response**: JSON in Shift-JIS encoding. Must decode from Shift-JIS to UTF-8 in client code.
- Response compression: gzip via Apache mod_deflate for `application/json` and `text/json` content types.

### 10.4 URL Encoding Requirements

Characters with special meaning in URLs must be percent-encoded:

| Character | Encoded |
|-----------|---------|
| `#` | `%23` |
| `+` | `%2B` |
| `/` | `%2F` |
| `:` | `%3A` |
| `=` | `%3D` |

This is particularly important for passwords that may contain these characters.

### 10.5 Serial Request Requirement

REQUEST I/F operates in a strict serial (one-at-a-time) mode:
- Send a request, wait for the response, then send the next request
- Parallel requests to the same virtual URL result in undefined behavior
- **Exception**: Master data download via `sUrlMaster` can run in parallel

### 10.6 Duplicate Request Prevention (p_no)

The `p_no` field prevents duplicate request processing:
- Initialize at login time
- Increment by 1 or more for each subsequent request
- Server rejects requests where `p_no <= previous p_no`
- This guards against browser/client automatic retry sending duplicate orders

### 10.7 Server Fault Tolerance

- API servers run on multiple parallel instances
- If one server fails, traffic is routed to surviving servers
- Requests in-flight during a server crash will not receive an API response (HTTPS-level error)
- After such failures, check order status via `CLMOrderListDetail` or the standard web interface

### 10.8 Insider Trading Restriction

Stocks declared for insider trading restrictions cannot be traded via API for new/correction orders.
Cancellation orders ARE permitted for insider-declared stocks (emergency consideration).
Use the standard web interface for new/correction orders on insider-declared stocks.

### 10.9 Second Password Requirement

The second password (`sSecondPassword`) is required for all order input operations (new, correct, cancel) regardless of the "password omission" setting in the standard web interface.

### 10.10 v4r8 POST Compatibility

When migrating from v4r7 to v4r8:
- Simply change the URL prefix from `e_api_v4r7` to `e_api_v4r8`
- Existing GET-based code continues to work without changes
- POST support is optional; use it when URL length limits are a concern

---

## 11. Provided Functions Summary

### 11.1 Auth Functions (Auth I/F)

| sCLMID | Function |
|--------|----------|
| `CLMAuthLoginRequest` / `CLMAuthLoginAck` | Authenticate and obtain virtual URLs |
| `CLMAuthLogoutRequest` / `CLMAuthLogoutAck` | Invalidate virtual URLs |

### 11.2 Business Functions (REQUEST I/F via sUrlRequest)

| sCLMID | Function |
|--------|----------|
| `CLMKabuNewOrder` | New stock order |
| `CLMKabuCorrectOrder` | Order correction/amendment |
| `CLMKabuCancelOrder` | Order cancellation |
| `CLMKabuCancelOrderAll` | Cancel all orders |
| `CLMGenbutuKabuList` | Spot stock holdings list |
| `CLMShinyouTategyokuList` | Margin positions list |
| `CLMZanKaiKanougaku` | Buying power |
| `CLMZanShinkiKanoIjiritu` | Margin capacity & maintenance ratio |
| `CLMZanUriKanousuu` | Sellable quantity |
| `CLMOrderList` | Order list |
| `CLMOrderListDetail` | Order/fill detail |
| `CLMZanKaiSummary` | Account summary |
| `CLMZanKaiKanougakuSuii` | Buying power history |
| `CLMZanKaiGenbutuKaitukeSyousai` | Spot stock purchase available amount detail |
| `CLMZanKaiSinyouSinkidateSyousai` | Margin new position available amount detail |
| `CLMZanRealHosyoukinRitu` | Real-time margin ratio |

### 11.3 Master Functions (REQUEST I/F via sUrlMaster)

| sCLMID | Function |
|--------|----------|
| `CLMEventDownload` | Master data download (streaming) |
| `CLMMfdsGetMasterData` | Master data query (v4r2+) |
| `CLMMfdsGetNewsHead` | News header query (v4r4+) |
| `CLMMfdsGetNewsBody` | News body query (v4r4+) |
| `CLMMfdsGetIssueDetail` | Issue detail query (v4r6+) |
| `CLMMfdsGetSyoukinZan` | Securities finance balance (v4r6+) |
| `CLMMfdsGetShinyouZan` | Margin balance query (v4r6+) |
| `CLMMfdsGetHibuInfo` | Reverse daily interest (v4r6+) |

#### Master Data Download Types (CLMEventDownload targets)

| sCLMID | Description |
|--------|-------------|
| `CLMSystemStatus` | System status |
| `CLMDateZyouhou` | Date information |
| `CLMYobine` | Tick size |
| `CLMUnyouStatus` | Operation status by state |
| `CLMUnyouStatusKabu` | Stock market operation status |
| `CLMUnyouStatusHasei` | Derivative market operation status |
| `CLMIssueMstKabu` | Stock issue master |
| `CLMIssueSizyouMstKabu` | Stock issue-market master |
| `CLMIssueSizyouKiseiKabu` | Stock issue-market regulations |
| `CLMIssueMstSak` | Futures issue master |
| `CLMIssueMstOp` | Options issue master |
| `CLMIssueSizyouKiseiHasei` | Derivative issue-market regulations |
| `CLMDaiyouKakeme` | Substitute collateral rate |
| `CLMHosyoukinMst` | Margin requirement master |
| `CLMOrderErrReason` | Exchange error/reason codes |
| `CLMEventDownloadComplete` | Download complete marker |

### 11.4 Market Price Functions (REQUEST I/F via sUrlPrice)

| sCLMID | Function |
|--------|----------|
| `CLMMfdsGetMarketPrice` | Market price query (max 120 tickers, v4r2+) |
| `CLMMfdsGetMarketPriceHistory` | Historical OHLCV query (~20 years, v4r3+) |

### 11.5 Event Functions (EVENT I/F via sUrlEvent or sUrlEventWebSocket)

| Event Type (`p_evt_cmd`) | Description | Connection |
|---------------------------|-------------|------------|
| `EC` — Order/Execution | Order status changes, fills, rejections | `sUrlEventWebSocket` with `p_evt_cmd=EC` |
| `ST` — Error Status | Error notification, then disconnect | Always delivered |
| `KP` — Keep-alive | Heartbeat every 5s if idle | Always delivered |
| `FD` — Market Price | Real-time stock prices (full snapshot + delta) | `p_evt_cmd=FD` |
| `NS` — News | Real-time news delivery | `p_evt_cmd=NS` |
| `SS` — System Status | Open/close notifications | `p_evt_cmd=SS` |
| `US` — Operation Status | Order acceptance start/end | `p_evt_cmd=US` |

**WebSocket connection**: `sUrlEventWebSocket` + query params `?p_rid=0&p_board_no=1000&p_eno=0&p_evt_cmd=EC` (see section 8 for full details).

**Data format**: Proprietary (NOT JSON). Field separator `^A` (0x01), key/value separator `^B` (0x02). See section 8.3.

---

## 12. Version History

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
GET https://kabuka.e-shiten.jp/e_api_v4r8/auth/?%7B%22sCLMID%22%3A%22CLMAuthLoginRequest%22%2C%22p_no%22%3A%221%22%2C%22p_sd_date%22%3A%222026.03.21-09%3A00%3A00.000%22%2C%22sUserId%22%3A%22MYID%22%2C%22sPassword%22%3A%22MYPASS%22%7D
```

Decoded JSON:
```json
{
  "sCLMID": "CLMAuthLoginRequest",
  "p_no": "1",
  "p_sd_date": "2026.03.21-09:00:00.000",
  "sUserId": "MYID",
  "sPassword": "MYPASS"
}
```

### Login Response

```json
{
  "sCLMID": "CLMAuthLoginAck",
  "sResultCode": "0",
  "sResultText": "",
  "sZyoutoekiKazeiC": "1",
  "sSecondPasswordOmit": "0",
  "sKinsyouhouMidokuFlg": "0",
  "sUrlRequest": "https://kabuka.e-shiten.jp/xxx/yyy/zzz/request/",
  "sUrlMaster": "https://kabuka.e-shiten.jp/xxx/yyy/zzz/master/",
  "sUrlPrice": "https://kabuka.e-shiten.jp/xxx/yyy/zzz/price/",
  "sUrlEvent": "https://kabuka.e-shiten.jp/xxx/yyy/zzz/event/",
  "sUrlEventWebSocket": "wss://kabuka.e-shiten.jp/xxx/yyy/zzz/event_ws/"
}
```

### New Order Request (CLMKabuNewOrder)

```json
{
  "sCLMID": "CLMKabuNewOrder",
  "sZyoutoekiKazeiC": "1",
  "sIssueCode": "7203",
  "sSizyouC": "00",
  "sBaibaiKubun": "3",
  "sCondition": "0",
  "sOrderPrice": "2500",
  "sOrderSuryou": "100",
  "sGenkinShinyouKubun": "0",
  "sOrderExpireDay": "0",
  "sGyakusasiOrderType": "0",
  "sGyakusasiZyouken": "0",
  "sGyakusasiPrice": "*",
  "sTatebiType": "*",
  "sTategyokuZyoutoekiKazeiC": "*",
  "sSecondPassword": "MY2NDPASS",
  "p_no": "2",
  "p_sd_date": "2026.03.21-09:00:01.000"
}
```

### New Order Response

```json
{
  "sCLMID": "CLMKabuNewOrder",
  "sResultCode": "0",
  "sResultText": "",
  "sWarningCode": "0",
  "sWarningText": "",
  "sOrderNumber": "27003158",
  "sEigyouDay": "20260321",
  "sOrderUkewatasiKingaku": "250000",
  "sOrderTesuryou": "0",
  "sOrderSyouhizei": "0",
  "sKinri": "-",
  "sOrderDate": "20260321090001"
}
```

### Order Detail Query Response

```json
{
  "sCLMID": "CLMOrderListDetail",
  "sResultCode": "0",
  "sOrderNumber": "27003158",
  "sEigyouDay": "20260321",
  "sIssueCode": "7203",
  "sOrderStatusCode": "10",
  "sOrderBaibaiKubun": "3",
  "sOrderOrderPrice": "2500",
  "sOrderOrderSuryou": "100",
  "sOrderCurrentSuryou": "0",
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
  "sOrderNumber": "27003158",
  "sIssueCode": "7203",
  "sOrderStatusCode": "10",
  "sYakuzyouPrice": "2500",
  "sYakuzyouSuryou": "100"
}
```
