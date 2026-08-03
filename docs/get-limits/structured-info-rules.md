# Structured Info Rules

## Limit rules

Limits must be represented consistently, even when providers report them differently.

If a source provides `used_percent`, the system should also calculate `remaining_percent` when possible.

If a source provides `remaining_percent`, the system should also calculate `used_percent` when possible.

If a source provides used, remaining, and total amounts, all available values should be preserved.

If only two amount values are available and the third can be calculated reliably, the system should calculate it.

`amount_unit` should describe what is being limited, for example `tokens`, `credits`, `usd`, `requests`, or another provider-specific unit.

## Time fields

`collected_at` is the time when `ai-limits` collected or read the source data.

`data_as_of` is the time when the source data itself was last current. For local files, transcripts, or hook payloads, this is usually the timestamp of the latest relevant source record or session. For live API or CLI responses, this may be the response or snapshot time.

Structured time fields may keep source-specific formats or UTC timestamps. User-facing surfaces convert them to the user's local time in the format documented in [presentation/time-display.md](../presentation/time-display.md).

The default terminal output uses `source` and `data_as_of` for the `Source {source}` line. It does not use `collected_at` for this line.

`usage.activity.latest_activity_at` is a separate business fact about user activity. It must not be treated as the default `Source {source}` timestamp unless it is also the best known timestamp for the source data itself.

## Subscription fields

`account.subscription_started_at` is when the user's current plan/subscription began. It is not the account creation date if the account existed on a different plan before.

`account.renewal_at` is the next billing/renewal date for the subscription itself. It is a separate business fact from `limits[].resets_at`, which is the automatic reset time of a rate-limit window, and from `available_limit_resets`, which is a manually redeemable reset count.

`account.price_amount` and `account.price_currency` carry the price as reported by the source, in the source's own currency. They must not be converted to another currency by the collection layer.

`account.price_note` must be filled with a short, user-readable disclaimer whenever the source does not guarantee that `price_amount`/`price_currency` is the price every user on that plan pays, for example because price can vary by country, region, currency, or active promotion. When the source-reported price is unconditional for the account being read, `price_note` may stay `null`.

`account.plan_management_url` and `account.billing_management_url` are optional deep links into the provider's own plan-change and billing-management pages. They must be `null` when the source does not expose a reliable link for the authenticated account; they must never be guessed or constructed from a generic provider marketing URL.

## Empty and unavailable values

If a value is not present in the source data, use `null`.

If a value exists but cannot be parsed reliably, use `null` and add a short explanation to `diagnostics`.

If a value can be calculated only by making a weak assumption, do not calculate it. Use `null` and add a short explanation to `diagnostics`.

If the source cannot be accessed, set `status.access_available` to `false`, `status.data_available` to `false`, and put the user-readable reason into `status.message`.

If the source is accessible but does not contain supported usage or limit data, set `status.access_available` to `true`, `status.data_available` to `false`, and put the reason into `status.message`.

If raw data can be returned for the source, set `raw_data_available` to `true`. If the implementation cannot expose raw data safely or technically, set it to `false`.
