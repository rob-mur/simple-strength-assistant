# Ubiquitous Language

## Training

| Term                    | Definition                                                                                           | Aliases to avoid                  |
| ----------------------- | ---------------------------------------------------------------------------------------------------- | --------------------------------- |
| **Set**                 | A single bout of an exercise with a recorded weight (or bodyweight), reps, and RPE                   | Rep group, effort                 |
| **Rep**                 | One complete repetition of an exercise movement within a Set                                         | Repetition                        |
| **RPE**                 | Rate of Perceived Exertion — a 1–10 scale (0.5 increments) indicating how close to failure a Set was | Effort, difficulty, intensity     |
| **Exercise**            | A named movement pattern with associated constraints (set type, rep range, weight increment)         | Movement, lift                    |
| **Training Day**        | All Sets logged on a single calendar date; the unit of session grouping (no explicit session record) | Session, workout                  |
| **Weighted Exercise**   | An Exercise where load is expressed as an external weight value                                      | Barbell exercise, loaded exercise |
| **Bodyweight Exercise** | An Exercise where load is the trainee's own body mass; progression is via reps, not weight           | BW exercise                       |

## Muscle Groups

| Term                    | Definition                                                                                                            | Aliases to avoid               |
| ----------------------- | --------------------------------------------------------------------------------------------------------------------- | ------------------------------ |
| **Muscle Group**        | A named anatomical grouping to which one or more Exercises contribute stimulus                                        | Body part, muscle, target      |
| **Muscle Group Weight** | A fractional value (0–1) expressing how much of an Exercise's stimulus is directed at a given Muscle Group            | Contribution, split, weighting |
| **Volume**              | The accumulated training stimulus delivered to a Muscle Group over a time period, measured in Intensity-Adjusted Sets | Work, load, tonnage            |

## Suggestions

| Term                | Definition                                                                                                                                                            | Aliases to avoid                     |
| ------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------ |
| **Suggestion**      | The algorithm's recommended weight, reps, and target RPE for the trainee's next Set                                                                                   | Prediction, recommendation, next set |
| **e1RM**            | Estimated one-rep maximum — the theoretical maximum weight for one rep, computed from a Set's weight, reps, and RPE using the linear-RPE exponential-rep formula      | 1RM, max, predicted max              |
| **Assumption**      | The formula output `(rpe × 0.03269803 + 0.6730197) × 0.970546521^(rep−1)` — the fraction of e1RM represented by a given reps-at-RPE combination                       | Percentage, fraction, coefficient    |
| **Historical e1RM** | The e1RM of the best Set (highest e1RM) logged for an Exercise within the history window, excluding the current Training Day                                          | Baseline, previous best              |
| **Today's e1RM**    | The e1RM of the most recently logged Set for an Exercise on the current Training Day; adapts to intra-session fatigue                                                 | Current e1RM, session e1RM           |
| **Peak e1RM**       | The highest e1RM across all Sets for an Exercise on a given Training Day; the data point used for trend calculation (distinct from Today's e1RM which tracks recency) | Daily max, session best              |
| **Blended e1RM**    | A weighted combination of Today's e1RM and Historical e1RM, controlled by the Today Blend Factor                                                                      | Adjusted e1RM, combined e1RM         |
| **Available PB**    | The headroom between the Suggestion weight and the trainee's previous best weight at the same rep count; zero means no personal best is on offer                      | PB delta, progression headroom       |
| **Infinite Mode**   | The default behaviour where the upper bound of the rep range extends one beyond the highest rep count ever performed, ensuring a fresh rep count is always available  | Uncapped mode                        |
| **No-Data State**   | The UI state shown when there is insufficient history to compute a meaningful Suggestion                                                                              | Empty state, cold start              |
| **Rep Range**       | The inclusive interval `[min_reps, max_reps]` within which Suggestions are generated; `max_reps` is nullable (Infinite Mode when null)                                | Rep window, rep limits               |

## Progress Detection

| Term                       | Definition                                                                                                                    | Aliases to avoid                      |
| -------------------------- | ----------------------------------------------------------------------------------------------------------------------------- | ------------------------------------- |
| **Progress State**         | The three-value signal emitted per Exercise: Progressing, Stalled, or Insufficient Data                                       | Progress signal, trend state          |
| **Progressing**            | The Progress State when the e1RM Trend slope is positive (> 0)                                                                | Improving, gaining, trending up       |
| **Stalled**                | The Progress State when the e1RM Trend slope is zero or negative (≤ 0); covers both flat and declining trajectories           | Plateaued, regressing, declining      |
| **Insufficient Data**      | The Progress State emitted when fewer than Min Sessions containing the Exercise exist within the Training Window              | No data, cold start, sparse           |
| **e1RM Trend**             | The slope of the linear regression fitted to the Peak e1RM series for an Exercise across Training Days in the Training Window | Progress curve, trendline, regression |
| **Intensity-Adjusted Set** | A single Set's contribution to Volume: `f(RPE) × Muscle Group Weight`, where `f(RPE) = RPE / 10`                              | Weighted set, effective set           |
| **Intensity Scalar**       | The value `f(RPE) = RPE / 10`; scales a Set's stimulus contribution by proximity to failure                                   | RPE weight, intensity factor, f(RPE)  |

## Settings

| Term                   | Definition                                                                                                                             | Aliases to avoid                        |
| ---------------------- | -------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------- |
| **Target RPE**         | The RPE the trainee is aiming to work at; used as the RPE input when projecting a Suggestion from e1RM                                 | Goal RPE, working RPE                   |
| **History Window**     | The number of days back the suggestion algorithm searches for Historical e1RM (default 30); scoped to per-set suggestion logic         | Lookback period, recency window         |
| **Today Blend Factor** | A 0–1 value controlling how much weight Today's e1RM has in the Blended e1RM (default 0.5); configurable to handle good/bad days       | Recency weight, blend ratio             |
| **Training Window**    | The rolling lookback period (in weeks, default 12) used for e1RM trend regression and Volume aggregation; distinct from History Window | Progress window, trend window, lookback |
| **Min Sessions**       | The minimum number of Training Days containing an Exercise required before a Progress State is emitted (default 3); configurable       | Data threshold, session threshold       |

## Relationships

- A **Training Day** contains one or more **Sets** across one or more **Exercises**.
- A **Set** belongs to exactly one **Exercise** and one **Training Day**.
- An **Exercise** has one or more **Muscle Group Weights** that sum to ≤ 1 (weighted contribution to each **Muscle Group**).
- A **Suggestion** is computed per **Exercise** using the **Blended e1RM**, **Rep Range**, and **Target RPE**.
- **Blended e1RM** = (Today's e1RM × Today Blend Factor) + (Historical e1RM × (1 − Today Blend Factor)).
- When only Today's e1RM is available (no historical Sets), the Blended e1RM equals Today's e1RM directly.
- When neither is available, the **No-Data State** is shown.
- **Infinite Mode** is active whenever `max_reps` is null on an **Exercise**.
- **Peak e1RM** is derived from the **Sets** of a **Training Day** and is the data point used in the **e1RM Trend** regression.
- **e1RM Trend** requires at least **Min Sessions** Training Days within the **Training Window** to produce a **Progress State**; otherwise **Insufficient Data** is returned.
- **Volume** for a **Muscle Group** on a given day = sum of **Intensity-Adjusted Sets** across all Sets whose Exercise contributes to that Muscle Group.

## Example dialogue

> **Dev:** "After the trainee logs a Set, how do we compute the next Suggestion?"

> **Domain expert:** "Take the Blended e1RM — that's Today's e1RM blended with the Historical e1RM at the Today Blend Factor. Then for each rep count in the Rep Range, project a weight using the Assumption at the Target RPE. Pick the rep count with the most Available PB."

> **Dev:** "What if they've already beaten the previous best at every rep count in the range?"

> **Domain expert:** "That's Infinite Mode — if there's no max_reps, we extend the upper bound to one above the highest rep count they've ever done. There's always a fresh rep count available."

> **Dev:** "And if it's a brand new exercise with no history at all?"

> **Domain expert:** "That's the No-Data State. Show a message — don't pretend to suggest something. If they've done Sets today but nothing older, skip the blend and use Today's e1RM directly."

> **Dev:** "How do we know if a trainee is actually getting stronger over time?"

> **Domain expert:** "We look at the e1RM Trend — a linear regression over the Peak e1RM for each Training Day that exercise appeared, within the Training Window. Positive slope means Progressing. Zero or negative means Stalled."

> **Dev:** "What if they've only done the exercise twice?"

> **Domain expert:** "If they have fewer Training Days than Min Sessions, we emit Insufficient Data — no guessing from two data points."

> **Dev:** "What about Volume — is that for tracking fatigue?"

> **Domain expert:** "Volume measures accumulated stimulus per Muscle Group. Each Set contributes one Intensity-Adjusted Set: the Intensity Scalar times the Muscle Group Weight. We aggregate daily, over 7 days for fatigue, and over the full Training Window for effort. Volume doesn't tell you if someone is progressing — that's the e1RM Trend's job."

## Flagged ambiguities

- **"Session"** was used loosely in the issue tracker and codebase (`WorkoutSession`, `session_id`) but the domain has no explicit session record — the unit is the **Training Day** (a calendar date). Code may use "session" internally but domain conversations should use Training Day. Note: **Min Sessions** is an unfortunate legacy name in the settings — it counts **Training Days**, not sessions.
- **"Prediction" vs "Suggestion"** — the Rust codebase uses `PredictedParameters` and `calculate_next_predictions`. The domain term is **Suggestion**; prediction implies a passive forecast, while a Suggestion implies an actionable recommendation the trainee can accept or override.
- **"History"** is ambiguous — it can mean raw logged Sets or specifically the Historical e1RM input to the algorithm. Prefer **History Set** for raw data and **Historical e1RM** when referring to the algorithm input.
- **"Today's e1RM" vs "Peak e1RM"** — both are e1RM values for a Training Day but serve different purposes. Today's e1RM is the _most recent_ set's e1RM (adapts to fatigue, used for in-session Suggestions). Peak e1RM is the _highest_ e1RM of the day (best performance, used as the trend data point). Never use "session e1RM" — it conflates the two.
- **"Intensity"** is overloaded: (1) in RPE context it means proximity to failure (a perceptual measure); (2) in **Intensity Scalar** / **Intensity-Adjusted Set** it means the numeric value `RPE / 10`. Avoid using "intensity" as a standalone noun — qualify it as RPE or use the compound terms above.
- **"History Window" vs "Training Window"** — both are lookback periods but for different purposes. **History Window** (days, default 30) scopes the suggestion algorithm's Historical e1RM search. **Training Window** (weeks, default 12) scopes progress detection and Volume aggregation. Never use "window" without the qualifying prefix.
