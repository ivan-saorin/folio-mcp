# Folio Financial Functions Specification

## Overview

Financial calculations for time value of money, loans, investments, depreciation, and portfolio analysis. All calculations use `Number` (BigRational) for precision. Exchange rates provided via external table reference (e.g., `Rates.USD_EUR`).

---

## Module Structure

```
folio-finance/
├── Cargo.toml
└── src/
    ├── lib.rs           # Registration
    ├── helpers.rs       # Common financial utilities
    ├── tvm.rs           # Time value of money
    ├── loans.rs         # Loan calculations
    ├── depreciation.rs  # Asset depreciation
    ├── bonds.rs         # Fixed income
    ├── returns.rs       # Investment returns
    └── rates.rs         # Interest rate conversions
```

---

## Day Count Conventions

Supported conventions for interest calculations:

| Convention | Days in Period | Days in Year | Code |
|------------|----------------|--------------|------|
| Actual/Actual | Actual | Actual | `"ACT/ACT"` |
| Actual/360 | Actual | 360 | `"ACT/360"` |
| Actual/365 | Actual | 365 | `"ACT/365"` |
| 30/360 | 30-day months | 360 | `"30/360"` |
| 30E/360 | European 30/360 | 360 | `"30E/360"` |

Default: `"ACT/365"` unless specified.

---

## Compounding Frequencies

| Code | Periods/Year |
|------|--------------|
| `"annual"` | 1 |
| `"semi"` | 2 |
| `"quarterly"` | 4 |
| `"monthly"` | 12 |
| `"weekly"` | 52 |
| `"daily"` | 365 |
| `"continuous"` | ∞ |

Default: `"annual"` unless specified.

---

## Functions

### Time Value of Money

#### `pv(rate, nper, pmt, [fv], [type])`

Present value of an annuity.

```markdown
| Metric | Formula                    | Result    |
|--------|----------------------------|-----------|
| pv     | pv(0.05, 10, -1000)        | 7721.73   |
| pv_due | pv(0.05, 10, -1000, 0, 1)  | 8107.82   |
```

**Parameters:**
- `rate`: Interest rate per period
- `nper`: Number of periods
- `pmt`: Payment per period (negative = outflow)
- `fv`: Future value (default: 0)
- `type`: 0 = end of period (default), 1 = beginning

#### `fv(rate, nper, pmt, [pv], [type])`

Future value of an annuity.

```markdown
| savings | fv(0.06/12, 240, -500, 0, 0) | 231,020.50 |
```

#### `npv(rate, cash_flows)`

Net present value of irregular cash flows.

```markdown
| flows | [-100000, 30000, 40000, 50000, 30000] |           |
| npv   | npv(0.10, flows)                       | 17,090.42 |
```

Cash flows assumed at end of each period, starting at period 1. Initial investment at period 0 should be added separately:

```markdown
| total_npv | flows[0] + npv(0.10, tail(flows)) | ... |
```

#### `xnpv(rate, cash_flows, dates)`

Net present value with specific dates.

```markdown
| flows | [-100000, 25000, 35000, 45000]                    |        |
| dates | [date(2024,1,1), date(2024,6,15), date(2025,1,1), date(2025,7,1)] | |
| xnpv  | xnpv(0.08, flows, dates)                          | 12,345 |
```

Uses ACT/365 day count.

#### `irr(cash_flows, [guess])`

Internal rate of return.

```markdown
| flows | [-100000, 30000, 40000, 50000, 30000] |        |
| irr   | irr(flows)                             | 0.1567 |
```

Uses Newton-Raphson. Default guess: 0.1. Returns error if no convergence.

#### `xirr(cash_flows, dates, [guess])`

IRR with specific dates.

```markdown
| xirr | xirr(flows, dates) | 0.1823 |
```

#### `mirr(cash_flows, finance_rate, reinvest_rate)`

Modified IRR (separates cost of capital from reinvestment rate).

```markdown
| mirr | mirr(flows, 0.10, 0.12) | 0.1345 |
```

---

### Loan Calculations

#### `pmt(rate, nper, pv, [fv], [type])`

Payment for a loan or annuity.

```markdown
| monthly_payment | pmt(0.05/12, 360, 250000) | -1342.05 |
```

Negative result = outflow.

#### `ppmt(rate, per, nper, pv, [fv], [type])`

Principal portion of a specific payment.

```markdown
| principal_pmt_1   | ppmt(0.05/12, 1, 360, 250000)   | -300.38  |
| principal_pmt_120 | ppmt(0.05/12, 120, 360, 250000) | -492.15  |
```

#### `ipmt(rate, per, nper, pv, [fv], [type])`

Interest portion of a specific payment.

```markdown
| interest_pmt_1   | ipmt(0.05/12, 1, 360, 250000)   | -1041.67 |
| interest_pmt_120 | ipmt(0.05/12, 120, 360, 250000) | -849.90  |
```

#### `nper(rate, pmt, pv, [fv], [type])`

Number of periods to pay off loan.

```markdown
| months | nper(0.05/12, -1500, 250000) | 294.5 |
```

#### `rate(nper, pmt, pv, [fv], [type], [guess])`

Interest rate per period.

```markdown
| rate   | rate(360, -1342.05, 250000) | 0.00417 |
| annual | rate * 12                   | 0.05    |
```

Uses Newton-Raphson iteration.

#### `amortization(rate, nper, pv, [periods_to_show])`

Full amortization schedule.

```markdown
| sched | amortization(0.05/12, 360, 250000, 12) | |
```

**Returns Object:**
```json
{
  "payment": Number,
  "total_interest": Number,
  "total_principal": Number,
  "schedule": [
    {
      "period": 1,
      "payment": 1342.05,
      "principal": 300.38,
      "interest": 1041.67,
      "balance": 249699.62
    },
    ...
  ]
}
```

`periods_to_show`: How many periods to include in schedule (default: all).

#### `cumipmt(rate, nper, pv, start_period, end_period, [type])`

Cumulative interest paid between periods.

```markdown
| total_int_yr1 | cumipmt(0.05/12, 360, 250000, 1, 12) | -12,387.45 |
```

#### `cumprinc(rate, nper, pv, start_period, end_period, [type])`

Cumulative principal paid between periods.

```markdown
| principal_yr1 | cumprinc(0.05/12, 360, 250000, 1, 12) | -3,717.15 |
```

---

### Depreciation

#### `sln(cost, salvage, life)`

Straight-line depreciation.

```markdown
| annual_dep | sln(100000, 10000, 10) | 9000 |
```

Formula: (cost - salvage) / life

#### `ddb(cost, salvage, life, period, [factor])`

Double declining balance.

```markdown
| yr1_dep | ddb(100000, 10000, 10, 1)    | 20000 |
| yr2_dep | ddb(100000, 10000, 10, 2)    | 16000 |
| yr5_dep | ddb(100000, 10000, 10, 5, 2) | 8192  |
```

Default factor: 2 (double). Use 1.5 for 150% declining.

#### `syd(cost, salvage, life, period)`

Sum-of-years-digits depreciation.

```markdown
| yr1_dep | syd(100000, 10000, 10, 1) | 16363.64 |
```

Formula: (cost - salvage) × (life - period + 1) / sum(1..life)

#### `vdb(cost, salvage, life, start_period, end_period, [factor], [no_switch])`

Variable declining balance with optional switch to straight-line.

```markdown
| dep_yr1_2 | vdb(100000, 10000, 10, 0, 2) | 36000 |
```

`no_switch`: If true, never switches to straight-line (default: false).

#### `depreciation_schedule(cost, salvage, life, method)`

Full depreciation schedule.

```markdown
| sched | depreciation_schedule(100000, 10000, 10, "ddb") | |
```

**Returns Object:**
```json
{
  "method": "ddb",
  "cost": 100000,
  "salvage": 10000,
  "life": 10,
  "schedule": [
    {"period": 1, "depreciation": 20000, "book_value": 80000},
    {"period": 2, "depreciation": 16000, "book_value": 64000},
    ...
  ],
  "total_depreciation": 90000
}
```

Methods: `"sln"`, `"ddb"`, `"syd"`, `"ddb150"` (1.5x declining).

---

### Bond Calculations

#### `bond_price(rate, yld, redemption, frequency, settlement, maturity, [day_count])`

Bond price per 100 face value.

```markdown
| price | bond_price(0.05, 0.06, 100, 2, date(2024,1,15), date(2034,1,15)) | 92.56 |
```

**Parameters:**
- `rate`: Annual coupon rate
- `yld`: Annual yield to maturity
- `redemption`: Redemption value per 100 face
- `frequency`: Coupon payments per year (1, 2, 4)
- `settlement`: Settlement date
- `maturity`: Maturity date
- `day_count`: Day count convention (default: "30/360")

#### `bond_yield(rate, price, redemption, frequency, settlement, maturity, [day_count], [guess])`

Yield to maturity.

```markdown
| ytm | bond_yield(0.05, 92.56, 100, 2, date(2024,1,15), date(2034,1,15)) | 0.06 |
```

#### `duration(rate, yld, frequency, settlement, maturity, [day_count])`

Macaulay duration (years).

```markdown
| dur | duration(0.05, 0.06, 2, date(2024,1,15), date(2034,1,15)) | 7.89 |
```

#### `mduration(rate, yld, frequency, settlement, maturity, [day_count])`

Modified duration.

```markdown
| mdur | mduration(0.05, 0.06, 2, date(2024,1,15), date(2034,1,15)) | 7.66 |
```

Formula: Macaulay duration / (1 + yld/frequency)

#### `convexity(rate, yld, frequency, settlement, maturity, [day_count])`

Bond convexity.

```markdown
| conv | convexity(0.05, 0.06, 2, date(2024,1,15), date(2034,1,15)) | 72.34 |
```

#### `accrint(issue, first_interest, settlement, rate, par, frequency, [day_count])`

Accrued interest.

```markdown
| accrued | accrint(date(2023,7,15), date(2024,1,15), date(2024,1,1), 0.05, 1000, 2) | 23.61 |
```

---

### Investment Returns

#### `cagr(start_value, end_value, years)`

Compound annual growth rate.

```markdown
| growth | cagr(10000, 25000, 5) | 0.2011 |
```

Formula: (end/start)^(1/years) - 1

#### `roi(gain, cost)`

Simple return on investment.

```markdown
| return | roi(5000, 20000) | 0.25 |
```

Formula: gain / cost

#### `holding_period_return(values)`

Total return over period from series of values.

```markdown
| values | [100, 105, 102, 110, 115] |        |
| hpr    | holding_period_return(values)   | 0.15   |
```

Formula: (final - initial) / initial

#### `annualized_return(total_return, years)`

Annualize a total return.

```markdown
| ann | annualized_return(0.50, 3) | 0.1447 |
```

Formula: (1 + total_return)^(1/years) - 1

#### `sharpe(returns, risk_free_rate)`

Sharpe ratio.

```markdown
| rets   | [0.05, 0.02, -0.01, 0.08, 0.03] |       |
| sharpe | sharpe(rets, 0.02)               | 0.567 |
```

Formula: (mean(returns) - risk_free) / stddev(returns)

#### `sortino(returns, risk_free_rate, [target])`

Sortino ratio (downside risk only).

```markdown
| sortino | sortino(rets, 0.02) | 0.823 |
```

Default target: 0 (any negative return is downside).

#### `max_drawdown(values)`

Maximum peak-to-trough decline.

```markdown
| values   | [100, 110, 95, 105, 90, 115] |        |
| mdd      | max_drawdown(values)          | -0.182 |
```

**Returns Object:**
```json
{
  "drawdown": -0.182,
  "peak_index": 1,
  "trough_index": 4,
  "peak_value": 110,
  "trough_value": 90,
  "recovery_index": 5
}
```

#### `calmar(values, years)`

Calmar ratio (CAGR / max drawdown).

```markdown
| calmar | calmar(values, 2) | 0.823 |
```

#### `volatility(returns, [annualize], [periods_per_year])`

Annualized volatility.

```markdown
| monthly_rets | [0.02, -0.01, 0.03, ...] |       |
| vol          | volatility(monthly_rets, true, 12) | 0.12 |
```

Default: annualize=true, periods_per_year=12.

#### `beta(asset_returns, market_returns)`

Beta coefficient.

```markdown
| beta | beta(stock_rets, sp500_rets) | 1.25 |
```

Formula: covariance(asset, market) / variance(market)

#### `alpha(asset_returns, market_returns, risk_free_rate)`

Jensen's alpha.

```markdown
| alpha | alpha(stock_rets, sp500_rets, 0.02) | 0.015 |
```

Formula: mean(asset) - (risk_free + beta × (mean(market) - risk_free))

#### `treynor(returns, market_returns, risk_free_rate)`

Treynor ratio.

```markdown
| treynor | treynor(stock_rets, sp500_rets, 0.02) | 0.08 |
```

Formula: (mean(returns) - risk_free) / beta

---

### Interest Rate Conversions

#### `effective_rate(nominal, periods)`

Convert nominal to effective annual rate.

```markdown
| ear | effective_rate(0.12, 12) | 0.1268 |
```

Formula: (1 + nominal/periods)^periods - 1

#### `nominal_rate(effective, periods)`

Convert effective to nominal rate.

```markdown
| nominal | nominal_rate(0.1268, 12) | 0.12 |
```

#### `continuous_rate(nominal, periods)`

Convert to continuously compounded rate.

```markdown
| cont | continuous_rate(0.12, 12) | 0.1194 |
```

Formula: periods × ln(1 + nominal/periods)

#### `discount_rate(future_value, present_value, periods)`

Implied discount rate.

```markdown
| rate | discount_rate(15000, 10000, 5) | 0.0845 |
```

#### `real_rate(nominal, inflation)`

Fisher equation: real interest rate.

```markdown
| real | real_rate(0.08, 0.03) | 0.0485 |
```

Formula: (1 + nominal) / (1 + inflation) - 1

---

## Implementation Notes

### Precision

All calculations use `Number` (BigRational):

```rust
fn pmt(rate: &Number, nper: &Number, pv: &Number, fv: &Number, type_: i32) -> Value {
    // For (1+r)^n, use arbitrary precision power
    let one = Number::one();
    let r_plus_1 = one.add(rate);
    let factor = r_plus_1.pow_rational(nper, ctx.precision);
    
    // Continue with BigRational arithmetic...
}
```

For IRR/XIRR Newton-Raphson:
- Max iterations: 100
- Tolerance: 1e-12 as BigRational
- Return error if no convergence

### Error Handling

```rust
// Rate must be > -1
if rate <= &Number::from_i64(-1) {
    return Value::Error(FolioError::domain_error(
        "Interest rate must be greater than -1"
    ));
}

// Number of periods must be positive
if nper <= &Number::zero() {
    return Value::Error(FolioError::domain_error(
        "Number of periods must be positive"
    ));
}

// Cash flows for IRR must have sign changes
if !has_sign_change(&flows) {
    return Value::Error(FolioError::domain_error(
        "IRR requires at least one positive and one negative cash flow"
    ));
}
```

### Date Handling

Bond and XNPV/XIRR functions use `FolioDateTime`:

```rust
fn year_fraction(d1: &FolioDateTime, d2: &FolioDateTime, convention: &str) -> Number {
    match convention {
        "ACT/365" => {
            let days = d2.diff_days(d1);
            Number::from_ratio(days, 365)
        }
        "ACT/360" => {
            let days = d2.diff_days(d1);
            Number::from_ratio(days, 360)
        }
        "30/360" => {
            // 30-day month convention
            let (y1, m1, d1) = d1.ymd();
            let (y2, m2, d2) = d2.ymd();
            let days = 360 * (y2 - y1) + 30 * (m2 - m1) + (d2.min(30) - d1.min(30));
            Number::from_ratio(days, 360)
        }
        _ => // default to ACT/365
    }
}
```

---

## Examples

### Mortgage Analysis

```markdown
## Mortgage @precision:10

| Metric          | Formula                                    | Result     |
|-----------------|--------------------------------------------|------------|
| principal       | 250000                                     |            |
| rate            | 0.065                                      |            |
| years           | 30                                         |            |
| monthly_rate    | rate / 12                                  |            |
| n_payments      | years * 12                                 |            |
| payment         | pmt(monthly_rate, n_payments, principal)   | -1580.17   |
| total_paid      | abs(payment) * n_payments                  | 568861.20  |
| total_interest  | total_paid - principal                     | 318861.20  |
| yr1_interest    | cumipmt(monthly_rate, n_payments, principal, 1, 12) | -16129.45 |
| yr1_principal   | cumprinc(monthly_rate, n_payments, principal, 1, 12) | -2832.59 |
```

### Investment Comparison

```markdown
## Investment Analysis

| Metric     | Formula                              | Result  |
|------------|--------------------------------------|---------|
| inv_a      | [-10000, 2000, 3000, 4000, 5000]     |         |
| inv_b      | [-10000, 1000, 2000, 3000, 8000]     |         |
| npv_a      | npv(0.08, tail(inv_a)) + inv_a[0]   | 1234.56 |
| npv_b      | npv(0.08, tail(inv_b)) + inv_b[0]   | 987.65  |
| irr_a      | irr(inv_a)                           | 0.1234  |
| irr_b      | irr(inv_b)                           | 0.1156  |
| better     | if(npv_a > npv_b, "A", "B")          | A       |
```

### Bond Pricing

```markdown
## Corporate Bond

| Metric     | Formula                                                        | Result |
|------------|----------------------------------------------------------------|--------|
| settle     | date(2024, 3, 15)                                              |        |
| mature     | date(2034, 3, 15)                                              |        |
| coupon     | 0.045                                                          |        |
| yield      | 0.052                                                          |        |
| price      | bond_price(coupon, yield, 100, 2, settle, mature)              | 94.23  |
| dur        | duration(coupon, yield, 2, settle, mature)                     | 8.12   |
| mdur       | mduration(coupon, yield, 2, settle, mature)                    | 7.91   |
| conv       | convexity(coupon, yield, 2, settle, mature)                    | 71.45  |
| price_chg  | -mdur * 0.01 + 0.5 * conv * 0.01^2                             | -0.076 |
```

---

## Function Summary

| Category | Functions |
|----------|-----------|
| **Time Value** | `pv`, `fv`, `npv`, `xnpv`, `irr`, `xirr`, `mirr` |
| **Loans** | `pmt`, `ppmt`, `ipmt`, `nper`, `rate`, `amortization`, `cumipmt`, `cumprinc` |
| **Depreciation** | `sln`, `ddb`, `syd`, `vdb`, `depreciation_schedule` |
| **Bonds** | `bond_price`, `bond_yield`, `duration`, `mduration`, `convexity`, `accrint` |
| **Returns** | `cagr`, `roi`, `holding_period_return`, `annualized_return`, `sharpe`, `sortino`, `max_drawdown`, `calmar`, `volatility`, `beta`, `alpha`, `treynor` |
| **Rates** | `effective_rate`, `nominal_rate`, `continuous_rate`, `discount_rate`, `real_rate` |

Total: 38 functions
