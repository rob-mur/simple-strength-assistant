# Quick Task: 25 - Fix Weight Suggestion Per Exercise

## Task Description

The user reported that the default weight when recording a set was the most recent set recorded globally, rather than being restricted per exercise. It should suggest the minimum weight the very first time an exercise is done, and from then on suggest the weight of the last set for that specific exercise.

## Root Cause

The `get_last_set_for_exercise` query correctly fetched the last set for the specific exercise. However, the `ActiveSession` UI component in Dioxus was retaining the input values (such as `weight_input`) when the user started a new session for a different exercise without completely remounting or successfully re-triggering the `use_effect` that was intended to sync the inputs. Because `use_effect(move || { let predicted = session.predicted; ... })` did not capture any reactive signals, it only ran once during the component's initial mount, leading to state bleeding across different exercises.

## Implementation Details

- Modified `src/app.rs` in `ActiveSession` component.
- Replaced the unreactive `use_effect` with a prop-to-signal sync pattern.
- Tracked `last_session_id` and `last_predicted` signals to forcefully update `reps_input`, `rpe_input`, and `weight_input` whenever the `session` prop updates (such as when starting a new session or after logging a set).
