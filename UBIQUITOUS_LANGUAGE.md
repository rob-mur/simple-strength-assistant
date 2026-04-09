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

## Suggestions

| Term                | Definition                                                                                                                                                           | Aliases to avoid                     |
| ------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------ |
| **Suggestion**      | The algorithm's recommended weight, reps, and target RPE for the trainee's next Set                                                                                  | Prediction, recommendation, next set |
| **e1RM**            | Estimated one-rep maximum — the theoretical maximum weight for one rep, computed from a Set's weight, reps, and RPE using the linear-RPE exponential-rep formula     | 1RM, max, predicted max              |
| **Assumption**      | The formula output `(rpe × 0.03269803 + 0.6730197) × 0.970546521^(rep−1)` — the fraction of e1RM represented by a given reps-at-RPE combination                      | Percentage, fraction, coefficient    |
| **Historical e1RM** | The e1RM of the best Set (highest e1RM) logged for an Exercise within the history window, excluding the current Training Day                                         | Baseline, previous best              |
| **Today's e1RM**    | The e1RM of the most recently logged Set for an Exercise on the current Training Day                                                                                 | Current e1RM, session e1RM           |
| **Blended e1RM**    | A weighted combination of Today's e1RM and Historical e1RM, controlled by the Today Blend Factor                                                                     | Adjusted e1RM, combined e1RM         |
| **Available PB**    | The headroom between the Suggestion weight and the trainee's previous best weight at the same rep count; zero means no personal best is on offer                     | PB delta, progression headroom       |
| **Infinite Mode**   | The default behaviour where the upper bound of the rep range extends one beyond the highest rep count ever performed, ensuring a fresh rep count is always available | Uncapped mode                        |
| **No-Data State**   | The UI state shown when there is insufficient history to compute a meaningful Suggestion                                                                             | Empty state, cold start              |
| **Rep Range**       | The inclusive interval `[min_reps, max_reps]` within which Suggestions are generated; `max_reps` is nullable (Infinite Mode when null)                               | Rep window, rep limits               |

## Settings

| Term                   | Definition                                                                                                                       | Aliases to avoid                |
| ---------------------- | -------------------------------------------------------------------------------------------------------------------------------- | ------------------------------- |
| **Target RPE**         | The RPE the trainee is aiming to work at; used as the RPE input when projecting a Suggestion from e1RM                           | Goal RPE, working RPE           |
| **History Window**     | The number of days back the algorithm searches for the Historical e1RM (default 30)                                              | Lookback period, recency window |
| **Today Blend Factor** | A 0–1 value controlling how much weight Today's e1RM has in the Blended e1RM (default 0.5); configurable to handle good/bad days | Recency weight, blend ratio     |

## Relationships

- A **Training Day** contains one or more **Sets** across one or more **Exercises**.
- A **Set** belongs to exactly one **Exercise** and one **Training Day**.
- A **Suggestion** is computed per **Exercise** using the **Blended e1RM**, **Rep Range**, and **Target RPE**.
- **Blended e1RM** = (Today's e1RM × Today Blend Factor) + (Historical e1RM × (1 − Today Blend Factor)).
- When only Today's e1RM is available (no historical Sets), the Blended e1RM equals Today's e1RM directly.
- When neither is available, the **No-Data State** is shown.
- **Infinite Mode** is active whenever `max_reps` is null on an **Exercise**.

## Example dialogue

> **Dev:** "After the trainee logs a Set, how do we compute the next Suggestion?"

> **Domain expert:** "Take the Blended e1RM — that's Today's e1RM blended with the Historical e1RM at the Today Blend Factor. Then for each rep count in the Rep Range, project a weight using the Assumption at the Target RPE. Pick the rep count with the most Available PB."

> **Dev:** "What if they've already beaten the previous best at every rep count in the range?"

> **Domain expert:** "That's Infinite Mode — if there's no max_reps, we extend the upper bound to one above the highest rep count they've ever done. There's always a fresh rep count available."

> **Dev:** "And if it's a brand new exercise with no history at all?"

> **Domain expert:** "That's the No-Data State. Show a message — don't pretend to suggest something. If they've done Sets today but nothing older, skip the blend and use Today's e1RM directly."

> **Dev:** "What's the difference between RPE and Target RPE?"

> **Domain expert:** "RPE is what the trainee _reported_ for a completed Set — it's an observation. Target RPE is a Setting — the intensity the algorithm aims for when projecting a Suggestion. They're separate concepts."

## Flagged ambiguities

- **"Session"** was used loosely in the issue tracker and codebase (`WorkoutSession`, `session_id`) but the domain has no explicit session record — the unit is the **Training Day** (a calendar date). Code may use "session" internally but domain conversations should use Training Day.
- **"Prediction" vs "Suggestion"** — the Rust codebase uses `PredictedParameters` and `calculate_next_predictions`. The domain term is **Suggestion**; prediction implies a passive forecast, while a Suggestion implies an actionable recommendation the trainee can accept or override.
- **"History"** is ambiguous — it can mean raw logged Sets or specifically the Historical e1RM input to the algorithm. Prefer **History Set** for raw data and **Historical e1RM** when referring to the algorithm input.
