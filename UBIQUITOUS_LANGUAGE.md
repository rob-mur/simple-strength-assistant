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

## Library & Plans

| Term                  | Definition                                                                                                                                 | Aliases to avoid                  |
| --------------------- | ------------------------------------------------------------------------------------------------------------------------------------------ | --------------------------------- |
| **Library**           | The user's collection of defined Exercises, browsable as a list and filterable between Active and Archived views                           | Catalog, exercise list            |
| **Active Exercise**   | An Exercise visible in the default Library view (`exercises.deleted_at IS NULL`); the Candidate Pool for new Plan Slots                    | Live exercise, current exercise   |
| **Archived Exercise** | An Exercise hidden from the default Library view (`exercises.deleted_at IS NOT NULL`); its historical Sets remain in the Historical Signal | Hidden exercise, deleted exercise |
| **Plan**              | An ordered list of Plan Slots representing one workout, with a lifecycle of Future → Active → Completed                                    | Workout, session, routine         |
| **Future Plan**       | A Plan not yet started (`started_at IS NULL`); editable scaffolding for an upcoming workout                                                | Scheduled workout, upcoming plan  |
| **Active Plan**       | A Plan started but not ended (`started_at IS NOT NULL AND ended_at IS NULL`); the workout the user is currently training                   | Live workout, in-progress plan    |
| **Completed Plan**    | A Plan whose workout has ended (`ended_at IS NOT NULL`); the historical record of one workout                                              | Finished workout, past plan       |
| **Plan Slot**         | A single Exercise's placement within a Plan, including its planned set count and ordering position                                         | Plan exercise, planned exercise   |
| **Record Screen**     | The UI surface where the user logs Sets for the currently selected Plan Slot in the Active Plan                                            | Workout view, training screen     |
| **Current Session**   | The Plan Slot in the Active Plan that the Record Screen is currently rendering; held in-memory as a snapshot                               | Active session, selected exercise |
| **Archive**           | The action of soft-deleting an Active Exercise: hides it from the Library, strips it from Future and Active Plans (reversible)             | Hide, remove                      |
| **Unarchive**         | The action of reverting an Archived Exercise to Active by clearing `deleted_at`; does not restore Plan Slots stripped during Archive       | Restore, unhide                   |
| **Permanent Delete**  | The action of irreversibly erasing an Exercise and cascading to its Sets, all Plan Slots, and any Plans that become empty as a result      | Purge, hard delete                |

## Muscle Groups

| Term                  | Definition                                                                                                                                                 | Aliases to avoid                   |
| --------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------- |
| **Muscle Group**      | A named anatomical grouping to which one or more Exercises contribute stimulus; one of 12 predefined groups, with optional hierarchy via parent/sub-muscle | Body part, muscle, target          |
| **Sub-Muscle**        | A more specific Muscle Group nested under a parent (e.g. Lateral Delts under Shoulders); users can tag at any level of the hierarchy                       | Sub-group, child muscle            |
| **Contribution Tier** | A discrete level expressing how much an Exercise stresses a Muscle Group: Primary (1.0), Secondary (0.5), or Tertiary (0.25)                               | Weight, split, weighting, priority |
| **Primary**           | The Contribution Tier (value 1.0) indicating a Muscle Group is a main target of the Exercise                                                               | Main, dominant                     |
| **Secondary**         | The Contribution Tier (value 0.5) indicating significant but non-primary stimulus to a Muscle Group                                                        | Supporting, accessory              |
| **Tertiary**          | The Contribution Tier (value 0.25) indicating minor involvement of a Muscle Group                                                                          | Minor, incidental                  |
| **Volume**            | The accumulated training stimulus delivered to a Muscle Group over a time period, measured in Intensity-Adjusted Sets                                      | Work, load, tonnage                |

## UI

| Term             | Definition                                                                                                | Aliases to avoid                    |
| ---------------- | --------------------------------------------------------------------------------------------------------- | ----------------------------------- |
| **Body Diagram** | The interactive front/back SVG of a human body used to select Muscle Groups by tapping anatomical regions | Body map, muscle picker, body chart |

## Suggestions

| Term                    | Definition                                                                                                                                                                                               | Aliases to avoid                     |
| ----------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------ |
| **Suggestion**          | The algorithm's recommended weight, reps, and target RPE for the trainee's next Set                                                                                                                      | Prediction, recommendation, next set |
| **e1RM**                | Estimated one-rep maximum — the theoretical maximum weight for one rep, computed from a Set's weight, reps, and RPE using the linear-RPE exponential-rep formula                                         | 1RM, max, predicted max              |
| **Assumption**          | The formula output `(rpe × 0.03269803 + 0.6730197) × 0.970546521^(rep−1)` — the fraction of e1RM represented by a given reps-at-RPE combination                                                          | Percentage, fraction, coefficient    |
| **Historical e1RM**     | The e1RM of the best Set (highest e1RM) logged for an Exercise within the History Window, excluding the current Training Day                                                                             | Baseline, previous best              |
| **Today's e1RM**        | The e1RM of the most recently logged Set for an Exercise on the current Training Day; adapts to intra-session fatigue                                                                                    | Current e1RM, session e1RM           |
| **Peak e1RM**           | The highest e1RM across all Sets for an Exercise on a given Training Day; the data point used for trend calculation (distinct from Today's e1RM which tracks recency)                                    | Daily max, session best              |
| **Blended e1RM**        | A weighted combination of Today's e1RM and Historical e1RM, controlled by the Today Blend Factor                                                                                                         | Adjusted e1RM, combined e1RM         |
| **Infinite Mode**       | The behaviour when `max_reps` is null: the upper bound of the rep range extends to one beyond the highest rep count recorded within the History Window, ensuring a fresh rep count is always available   | Uncapped mode                        |
| **No-Data State**       | The fallback state when there is insufficient history to compute a meaningful Suggestion; cold-start defaults are used instead                                                                           | Empty state, cold start              |
| **Rep Range**           | The inclusive interval `[min_reps, max_reps]` within which Suggestions are generated; `max_reps` is nullable (Infinite Mode when null)                                                                   | Rep window, rep limits               |
| **RIR**                 | Reps in Reserve — the standard linear approximation of remaining capacity: `RIR = 10 − RPE`; RPE 10 = 0 RIR, RPE 9 = 1 RIR, etc.                                                                         | Reps left, buffer                    |
| **Failure Reps**        | The estimated maximum reps achievable to failure for a Bodyweight Set: `reps_done + RIR`; the bodyweight analogue of e1RM — a rep-count measure of capacity                                              | Max reps, rep max                    |
| **Historical Max at R** | The maximum weight recorded within the History Window in a Set where `reps_done ≥ R`; used to determine whether a Suggestion offers a personal best at rep count R                                       | Rep PB, per-rep best                 |
| **Per-Rep PB Margin**   | `projected_weight(blended_e1rm, r, target_rpe) − Historical Max at R`; positive means a personal best is available at rep count R; in bounded mode, the rep with the highest positive margin is selected | PB headroom, margin                  |
| **Clamped Suggestion**  | A Suggestion whose rep count has been constrained to a Rep Range boundary; surfaced in the UI so the trainee knows the Rep Range is limiting the recommendation                                          | Bounded suggestion, constrained reps |
| **Candidate Pool**      | The set of Active Exercises eligible to be recommended for new Plan Slots; Archived Exercises are excluded                                                                                               | Eligible set, available exercises    |
| **Historical Signal**   | The set of completed Sets that feeds the suggestion algorithm and Volume aggregation; includes Sets logged against Archived Exercises                                                                    | Training data, signal set            |

## Progress Detection

| Term                       | Definition                                                                                                                                                                                                                | Aliases to avoid                      |
| -------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------- |
| **Progress State**         | The three-value signal emitted per Exercise: Progressing, Stalled, or Insufficient Data                                                                                                                                   | Progress signal, trend state          |
| **Progressing**            | The Progress State when the e1RM Trend slope is positive (> 0)                                                                                                                                                            | Improving, gaining, trending up       |
| **Stalled**                | The Progress State when the e1RM Trend slope is zero or negative (≤ 0); covers both flat and declining trajectories                                                                                                       | Plateaued, regressing, declining      |
| **Insufficient Data**      | The Progress State emitted when fewer than Min Sessions containing the Exercise exist within the Training Window                                                                                                          | No data, cold start, sparse           |
| **e1RM Trend**             | The slope of the linear regression fitted to the Peak e1RM series for an Exercise across Training Days in the Training Window                                                                                             | Progress curve, trendline, regression |
| **Intensity-Adjusted Set** | A single Set's contribution to Volume: `f(RPE) × tier_value`, where `f(RPE) = RPE / 10` and the tier value is the **Contribution Tier** of the Exercise for that Muscle Group (Primary=1.0, Secondary=0.5, Tertiary=0.25) | Weighted set, effective set           |
| **Intensity Scalar**       | The value `f(RPE) = RPE / 10`; scales a Set's stimulus contribution by proximity to failure                                                                                                                               | RPE weight, intensity factor, f(RPE)  |

## Settings

| Term                        | Definition                                                                                                                                                                                  | Aliases to avoid                        |
| --------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------- |
| **Target RPE**              | The RPE the trainee is aiming to work at; used as the RPE input when projecting a Suggestion from e1RM                                                                                      | Goal RPE, working RPE                   |
| **History Window**          | The number of days back the suggestion algorithm searches for Historical e1RM (default 30); scoped to per-set suggestion logic                                                              | Lookback period, recency window         |
| **Today Blend Factor**      | A 0–1 value controlling how much weight Today's e1RM has in the Blended e1RM (default 0.5); configurable to handle good/bad days                                                            | Recency weight, blend ratio             |
| **Training Window**         | The rolling lookback period, configured in **weeks** (default 12), used for e1RM trend regression and Volume aggregation; exposed as weeks in the settings UI; distinct from History Window | Progress window, trend window, lookback |
| **Min Sessions**            | The minimum number of Training Days containing an Exercise required before a Progress State is emitted (default 3); configurable                                                            | Data threshold, session threshold       |
| **Default Bodyweight Reps** | The rep count used as the cold-start Suggestion for Bodyweight Exercises when no history exists; configurable in settings (default 10)                                                      | Default reps, starting reps             |

## Relationships

- A **Training Day** contains one or more **Sets** across one or more **Exercises**.
- A **Set** belongs to exactly one **Exercise** and one **Training Day**.
- An **Exercise** has one or more **Muscle Groups**, each assigned a **Contribution Tier** (Primary, Secondary, or Tertiary).
- A **Muscle Group** may have **Sub-Muscles**; users can tag at any level of the hierarchy.
- A **Suggestion** is computed per **Exercise** using the **Blended e1RM**, **Rep Range**, and **Target RPE**.
- **Blended e1RM** = (Today's e1RM × Today Blend Factor) + (Historical e1RM × (1 − Today Blend Factor)); applies to both weighted (e1RM values) and bodyweight (Failure Reps values) exercises, using the same Today Blend Factor setting.
- When only Today's e1RM is available (no historical Sets), the Blended e1RM equals Today's e1RM directly; same applies to Failure Reps for Bodyweight Exercises.
- When neither is available, the **No-Data State** is active: for Weighted Exercises the last session's weight is carried forward; for Bodyweight Exercises **Default Bodyweight Reps** is used.
- For a **Weighted Exercise**, the suggested rep count is the one with the highest positive **Per-Rep PB Margin** in the Rep Range; if no positive margin exists and **Infinite Mode** is active, the next uncovered rep count (`max_data_rep + 1`) is suggested; in bounded mode with no positive margin, the least-negative margin is used.
- **Historical Max at R** = max weight in the History Window across all Sets where `reps_done ≥ R`.
- For a **Bodyweight Exercise**, the suggested reps = `round(blended_failure_reps − (10 − Target RPE))`, where **Failure Reps** = `reps_done + (10 − RPE)` for each set.
- A **Clamped Suggestion** occurs when the raw suggested reps fall outside the **Rep Range**; the rep count is constrained to `min_reps` or `max_reps` and the UI signals this to the trainee.
- **Infinite Mode** is active whenever `max_reps` is null on an **Exercise**.
- **Peak e1RM** is derived from the **Sets** of a **Training Day** and is the data point used in the **e1RM Trend** regression.
- **e1RM Trend** requires at least **Min Sessions** Training Days within the **Training Window** to produce a **Progress State**; otherwise **Insufficient Data** is returned.
- **Volume** for a **Muscle Group** on a given day = sum of **Intensity-Adjusted Sets** across all Sets whose Exercise contributes to that Muscle Group.
- A **Plan** contains zero or more **Plan Slots**, each referencing one **Exercise**.
- A **Plan Slot**'s **Exercise** may be **Active** or **Archived**; archiving an Exercise strips its Slots from **Future Plans** and the **Active Plan** only — **Completed Plan** Slots are untouched.
- The **Current Session** is always a Plan Slot within the **Active Plan**.
- The **Candidate Pool** for **Suggestions** filters on **Active Exercise**; the **Historical Signal** does not — a recently archived Exercise's prior Sets continue to inform Volume and the **e1RM Trend**.
- **Archive** is reversible at the Exercise level (via **Unarchive**) but irreversible at the Plan level: stripped Plan Slots and emptied **Future Plans** are not restored on Unarchive.
- **Permanent Delete** cascades: the Exercise's **Sets** are soft-deleted; all of its **Plan Slots** (including those in **Completed Plans**) are soft-deleted; any **Plan** whose remaining Slots are all soft-deleted is itself soft-deleted.

## Example dialogue

> **Dev:** "After the trainee logs a Set, how do we compute the next Suggestion?"

> **Domain expert:** "Take the Blended e1RM — that's Today's e1RM blended with the Historical e1RM at the Today Blend Factor. Then for each rep count in the Rep Range, compute the Per-Rep PB Margin: projected weight minus the Historical Max at that rep count. Pick the rep count with the highest positive margin."

> **Dev:** "What counts as the Historical Max at, say, 3 reps?"

> **Domain expert:** "The maximum weight recorded within the History Window in a Set where the trainee did at least 3 reps. A 4-rep Set at 100 kg covers reps 1 through 4."

> **Dev:** "What if the Blended e1RM is low that day and no rep count has a positive Per-Rep PB Margin?"

> **Domain expert:** "In Infinite Mode, we extend one beyond the highest rep count recorded — there's always an uncovered rep count. In bounded mode, we pick the least-negative margin: best available performance for a down day. If the raw suggestion falls outside the Rep Range we clamp it and mark it a Clamped Suggestion so the trainee knows."

> **Dev:** "How does this work for a Bodyweight Exercise?"

> **Domain expert:** "Instead of e1RM we use Failure Reps — reps done plus RIR, where RIR is 10 minus RPE. We blend today's and historical Failure Reps using the same Today Blend Factor, then subtract the RIR at Target RPE to get the suggested rep count."

> **Dev:** "And if it's a brand new exercise with no history at all?"

> **Domain expert:** "That's the No-Data State. For weighted, carry forward the weight from the last session. For bodyweight, use Default Bodyweight Reps."

> **Dev:** "How do we know if a trainee is actually getting stronger over time?"

> **Domain expert:** "We look at the e1RM Trend — a linear regression over the Peak e1RM for each Training Day that exercise appeared, within the Training Window. Positive slope means Progressing. Zero or negative means Stalled."

> **Dev:** "What if they've only done the exercise twice?"

> **Domain expert:** "If they have fewer Training Days than Min Sessions, we emit Insufficient Data — no guessing from two data points."

> **Dev:** "What about Volume — is that for tracking fatigue?"

> **Domain expert:** "Volume measures accumulated stimulus per Muscle Group. Each Set contributes one Intensity-Adjusted Set: the Intensity Scalar times the Contribution Tier value. A Primary exercise counts fully, Secondary at half, Tertiary at a quarter. We aggregate daily, over 7 days for fatigue, and over the full Training Window for effort. Volume doesn't tell you if someone is progressing — that's the e1RM Trend's job."

> **Dev:** "When a user creates a new Exercise, how do they assign Muscle Groups?"
>
> **Domain expert:** "They tap regions on the Body Diagram — it's a front/back SVG of a human body. Each tap toggles a Muscle Group and creates a card with a Contribution Tier defaulting to Primary. They can tap the card to cycle through Secondary, Tertiary, or remove it entirely."
>
> **Dev:** "What if they only pick one Muscle Group?"
>
> **Domain expert:** "Then it's automatically Primary — no extra step. The rule is: at least one Muscle Group is required. You can't save an Exercise without one."
>
> **Dev:** "And if they edit the Muscle Groups later?"
>
> **Domain expert:** "Fully editable, and changes are retroactive. No snapshotting — if you fix a tagging mistake, the whole system reflects the update."

> **Dev:** "If the trainee Archives an **Exercise**, what happens to their **Plans** and history?"
>
> **Domain expert:** "The Exercise becomes an **Archived Exercise** — hidden from the Library by default, but findable via the 'Show archived' toggle. Its **Plan Slots** are stripped from any **Future Plans** and the **Active Plan**, and any Future Plan that becomes empty is deleted. **Completed Plans** are untouched, and all the **Sets** logged against it remain in the **Historical Signal** so trends and Volume aren't affected."
>
> **Dev:** "And the **Candidate Pool** for new Plans?"
>
> **Domain expert:** "An Archived Exercise drops out of the Candidate Pool immediately — it can't be added to a new Plan Slot until **Unarchived**. The asymmetry is deliberate: the engine forgets it as a recommendation candidate while still remembering its training history."
>
> **Dev:** "What if they meant to **Permanent Delete** instead of Archive?"
>
> **Domain expert:** "There's an escalation link in the Archive dialog. Permanent Delete is irreversible — it cascades to all Sets, all Plan Slots including those in Completed Plans, and any Plan that becomes empty as a result. The dialog shows the count up front."

## Flagged ambiguities

- **"Session"** was used loosely in the issue tracker and codebase (`WorkoutSession`, `session_id`) but the domain has no explicit session record — the unit is the **Training Day** (a calendar date). Code may use "session" internally but domain conversations should use Training Day. Note: **Min Sessions** is an unfortunate legacy name in the settings — it counts **Training Days**, not sessions.
- **"Prediction" vs "Suggestion"** — the Rust codebase uses `PredictedParameters` and `calculate_next_predictions`. The domain term is **Suggestion**; prediction implies a passive forecast, while a Suggestion implies an actionable recommendation the trainee can accept or override.
- **"History"** is ambiguous — it can mean raw logged Sets or specifically the Historical e1RM input to the algorithm. Prefer **History Set** for raw data and **Historical e1RM** when referring to the algorithm input.
- **"Today's e1RM" vs "Peak e1RM"** — both are e1RM values for a Training Day but serve different purposes. Today's e1RM is the _most recent_ set's e1RM (adapts to fatigue, used for in-session Suggestions). Peak e1RM is the _highest_ e1RM of the day (best performance, used as the trend data point). Never use "session e1RM" — it conflates the two.
- **"Intensity"** is overloaded: (1) in RPE context it means proximity to failure (a perceptual measure); (2) in **Intensity Scalar** / **Intensity-Adjusted Set** it means the numeric value `RPE / 10`. Avoid using "intensity" as a standalone noun — qualify it as RPE or use the compound terms above.
- **"History Window" vs "Training Window"** — both are lookback periods but for different purposes. **History Window** (days, default 30) scopes the suggestion algorithm's Historical e1RM search. **Training Window** (weeks, default 12) scopes progress detection and Volume aggregation. Never use "window" without the qualifying prefix.
- **"Muscle Group Weight" (deprecated)** — the original design used freeform relative weights normalised at calculation time. This has been replaced by **Contribution Tier** (discrete Primary/Secondary/Tertiary). Do not use "weight" to describe an Exercise's relationship to a Muscle Group — use **Contribution Tier** or the specific tier name.
- **Contribution Tier vs Muscle Group Weight (normalisation)** — the old Muscle Group Weight system normalised stimulus to always sum to 100% across Muscle Groups. Contribution Tiers do not: each Muscle Group receives its full tier value independently. Volume totals will therefore be higher for multi-muscle exercises under the new model.
- **"Available PB" (deprecated)** — superseded by **Per-Rep PB Margin**. Available PB implied a binary has/hasn't framing; Per-Rep PB Margin is the signed value actually used for rep selection. Do not use Available PB in new code or discussion.
- **"Max reps" for Bodyweight Exercises** — avoid "max reps" as a standalone term; it ambiguously refers to either the `max_reps` Rep Range bound or **Failure Reps** (the capacity estimate). Use **Failure Reps** for the estimated maximum and **Rep Range** bound for the configuration.
- **"Workout" vs "Plan" vs "Training Day"** — three related concepts used loosely. **Plan** is the user-managed list of Plan Slots to be executed (a row in `workout_plans`). **Training Day** is the analytical calendar grouping of all Sets logged on one date. They overlap but are not synonymous: a Training Day spans whatever Sets were logged that day regardless of which Plans they came from. Avoid "Workout" as a domain term — use **Plan** for the executable construct and **Training Day** for the analytical unit.
- **"Active"** is overloaded. **Active Exercise** = not Archived (`exercises.deleted_at IS NULL`). **Active Plan** = started but not ended (`started_at IS NOT NULL AND ended_at IS NULL`). Always qualify with the noun. "Active" alone is ambiguous in any conversation that touches both Exercises and Plans.
- **"Delete"** is overloaded between **Archive** (safe, reversible soft-delete of an Exercise) and **Permanent Delete** (irreversible cascade through Sets, Plan Slots, and emptied Plans). User-facing copy must distinguish the two; "delete" alone in UI is reserved for the destructive action — the safe action is "Archive".
- **`deleted_at` semantics** — the column is used uniformly across `exercises`, `workout_plans`, `workout_plan_exercises`, and `completed_sets` to mean "soft-deleted, hidden from active queries." For `exercises` specifically, a non-null `deleted_at` corresponds to the domain term **Archived**; for the other tables it corresponds to Archive cascade fallout or Permanent Delete cascade fallout. The column name does not distinguish which. The asymmetry between domain term ("Archived") and column name (`deleted_at`) is intentional — no migration.
- **Candidate Pool vs Historical Signal asymmetry** — when querying for Suggestions, the **Candidate Pool** must filter `e.deleted_at IS NULL` so Archived Exercises aren't recommended; the **Historical Signal** must NOT add this filter to its JOIN onto `exercises`, so Archived Exercises' prior Sets continue to inform Volume and trend calculations. New analytical queries should be reviewed against this rule.
