# Feature Research

**Domain:** Exercise Library for Workout Tracking PWA
**Researched:** 2026-03-01
**Confidence:** MEDIUM

## Feature Landscape

### Table Stakes (Users Expect These)

Features users assume exist. Missing these = product feels incomplete.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| Exercise list view | All workout apps provide a way to browse exercises | LOW | Simple list rendering from existing database |
| Exercise search by name | Users expect instant search, not endless scrolling | LOW | Text filter on exercise names already in DB |
| Last performed date | Progressive overload requires knowing "what did I do last time" | LOW | Query exercises table for most recent workout_session |
| Total sessions count | Users track consistency - "I've benched 47 times this year" | LOW | COUNT query on completed sets grouped by exercise |
| Edit exercise name | Users fix typos or adjust naming conventions | MEDIUM | UPDATE query with validation to prevent duplicates |
| Previous session stats | See last weight/reps to plan progressive overload | MEDIUM | JOIN exercises with most recent completed_sets |
| Archive exercise (not delete) | Hide exercises without orphaning workout history | MEDIUM | Boolean flag to filter views, preserves referential integrity |

### Differentiators (Competitive Advantage)

Features that set the product apart. Not required, but valuable.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| Offline-first with no cloud | User owns data locally, no account/subscription required | LOW | Already core to app philosophy (v1.0 foundation) |
| Performance indicators (volume trend) | Show if user is getting stronger without manual analysis | MEDIUM | Calculate total volume trend over last N sessions |
| Personal records (PR) tracking | Auto-detect and highlight PRs (heaviest weight, most reps) | MEDIUM | MAX queries across exercise history with date stamps |
| Exercise usage heatmap | Visual of most/least trained exercises over time | HIGH | Requires date-based aggregation and visualization |
| Quick-add from library to active workout | Tap exercise in library → immediately added to current session | LOW | Leverages existing workout session creation flow |

### Anti-Features (Commonly Requested, Often Problematic)

Features that seem good but create problems.

| Feature | Why Requested | Why Problematic | Alternative |
|---------|---------------|-----------------|-------------|
| Exercise categorization by muscle group | Users want to filter by "chest" or "legs" | Adds complexity without validation - user won't maintain tags | Use search by exercise name - users already know what they're looking for |
| Pre-populated exercise database | Saves users from creating exercises manually | Wrong exercise names for user's preference, bloats initial load | Let users create exercises naturally during workouts, build personal library |
| Exercise deletion | "I don't do squats anymore, delete it" | Orphans workout history, breaks referential integrity | Archive instead - hides from library but preserves data |
| Exercise video/image tutorials | "Show me how to do a Romanian deadlift" | Massive storage overhead, copyright issues, scope creep | User can Google form - library is for tracking not instruction |
| Detailed form cues and notes | "Track technique reminders per exercise" | Clutters UI during workout, users ignore after novelty wears off | Keep library lean - focus on stats not instruction |
| Exercise templates with rep/weight suggestions | "Auto-fill my bench press sets based on history" | Belongs in workout prescription, not exercise library | Defer to future prescription milestone |

## Feature Dependencies

```
[Exercise List View]
    └──requires──> [Database Schema for Exercises] ✓ (exists)
                       └──requires──> [Completed Sets Table] ✓ (exists)

[Search by Name]
    └──requires──> [Exercise List View]

[Last Performed Date]
    └──requires──> [Exercise List View]
    └──requires──> [JOIN with workout_sessions table]

[Previous Session Stats]
    └──requires──> [Last Performed Date]
    └──requires──> [Completed Sets Data] ✓ (exists)

[Edit Exercise Name]
    └──requires──> [Exercise List View]

[Archive Exercise]
    └──requires──> [Exercise List View]
    └──requires──> [Database Schema Update] (add archived boolean)

[Performance Indicators]
    └──requires──> [Previous Session Stats]
    └──enhances──> [Exercise List View]

[Personal Records Tracking]
    └──requires──> [Previous Session Stats]
    └──enhances──> [Exercise List View]

[Quick-add to Workout]
    └──requires──> [Active Workout Session] ✓ (exists)
    └──enhances──> [Exercise List View]
```

### Dependency Notes

- **Exercise List View requires Database Schema:** Exercise metadata already exists from v1.0 workout logging
- **Last Performed Date requires JOIN:** Need to query workout_sessions table to find most recent session containing each exercise
- **Previous Session Stats requires Last Performed:** Can't show stats without knowing which session was last
- **Archive requires Schema Update:** Need to add `archived` boolean column to exercises table
- **Performance Indicators enhances Exercise List:** Adds value but doesn't block basic functionality
- **Quick-add enhances Exercise List:** Creates shortcut from library to active workout flow

## MVP Definition

### Launch With (v1.1 - Exercise Library Tab)

Minimum viable product — what's needed to validate the concept.

- [x] **Exercise List View** — Table stakes, users need to see what exercises they've created
- [x] **Search by Name** — Table stakes, prevents endless scrolling (complexity is LOW, no excuse to skip)
- [x] **Last Performed Date** — Table stakes for progressive overload tracking
- [x] **Total Sessions Count** — Table stakes for consistency tracking
- [x] **Edit Exercise Name** — Table stakes, users need to fix typos
- [x] **Archive Exercise** — Table stakes, safer than deletion and preserves data integrity
- [ ] **Previous Session Stats (weight/reps from last time)** — Differentiator, but high value for progressive overload

### Add After Validation (v1.x)

Features to add once core library is working.

- [ ] **Personal Records (PR) Tracking** — Add when users request "what's my best bench press?"
- [ ] **Performance Indicators (volume trend)** — Add when users want to see strength trends without manual analysis
- [ ] **Quick-add from Library to Workout** — Add when users request faster way to add exercises mid-workout

### Future Consideration (v2+)

Features to defer until product-market fit is established.

- [ ] **Exercise Usage Heatmap** — Complex visualization, defer until users request insights on training balance
- [ ] **Exercise Templates with Auto-suggestions** — Belongs in workout prescription milestone, not library
- [ ] **Muscle Group Categorization** — Currently out of scope per PROJECT.md, defer indefinitely unless users demand it

## Feature Prioritization Matrix

| Feature | User Value | Implementation Cost | Priority |
|---------|------------|---------------------|----------|
| Exercise List View | HIGH | LOW | P1 |
| Search by Name | HIGH | LOW | P1 |
| Last Performed Date | HIGH | LOW | P1 |
| Total Sessions Count | HIGH | LOW | P1 |
| Edit Exercise Name | HIGH | MEDIUM | P1 |
| Archive Exercise | HIGH | MEDIUM | P1 |
| Previous Session Stats | HIGH | MEDIUM | P1 |
| Personal Records Tracking | MEDIUM | MEDIUM | P2 |
| Performance Indicators | MEDIUM | MEDIUM | P2 |
| Quick-add to Workout | MEDIUM | LOW | P2 |
| Exercise Usage Heatmap | LOW | HIGH | P3 |

**Priority key:**
- P1: Must have for launch (v1.1)
- P2: Should have, add when possible (v1.x)
- P3: Nice to have, future consideration (v2+)

## Competitor Feature Analysis

| Feature | Strong/Hevy/JEFIT Pattern | Our Approach | Rationale |
|---------|---------------------------|--------------|-----------|
| Exercise Database | Pre-populated with 900-1400+ exercises | User-created only, build personal library | Avoids bloat, respects user's naming preferences, aligns with offline-first philosophy |
| Exercise Search | Autocomplete with filters (muscle group, equipment, difficulty) | Simple text search by name | Keep simple for v1.1, users already know exercise names from their workouts |
| Exercise History | "Last performed" + graphs of volume/1RM over time | Last performed date + previous session stats (weight/reps) | Graph trends deferred to v1.x, focus on immediate progressive overload needs |
| Exercise Editing | Edit name, notes, muscle groups, equipment tags | Edit name only, archive instead of delete | Avoid feature bloat, muscle groups out of scope |
| Exercise Organization | Categories by muscle group, sortable by name/recently used | List view with search, sortable by name/last performed | Simpler mental model, no category maintenance burden |
| Personal Records | Auto-tracked PRs with notifications/badges | Auto-tracked PRs (no notifications) | P2 feature, notifications would require complex state management |
| Quick Actions | Swipe to delete/edit, long-press for options | Tap for details, edit/archive buttons in detail view | Mobile-friendly but avoid gesture-heavy UX until validated |

## User Experience Patterns (From Research)

### Critical UX Insights

**Minimize Friction:**
- Workout apps should limit actions to 3 steps max - 40% of users abandon apps with bland logging experiences
- Large, easy-to-tap elements essential for gym environment
- Distraction-free design: no pop-ups, no nav bar clutter during workouts

**Progressive Overload Support:**
- Most apps display previous performance (weight/reps/sets) when logging new workout
- This information should be visible during workout, not buried in analytics
- Users need to see "last time I did 3x8 @ 135lbs" to plan today's session

**Data That Matters:**
- Apps calculate estimated 1RM, track personal records automatically, show progression graphs
- Total volume tracking (weight × reps × sets) is standard for monitoring progress
- Quick logging in under 3 seconds per set is expected

**Common Pitfalls to Avoid:**
- Over 55% of users abandon fitness apps within 90 days due to complexity
- Exercise libraries with 1400+ exercises sound impressive but users only use ~30 regularly
- Tracking too many variables (rest time, tempo, perceived exertion, etc.) causes abandonment
- Battery drain from heavy background processing frustrates users

### Exercise Library Specific Patterns

**List Organization:**
- Resources shouldn't be "piled up" causing endless scrolling
- Divide by difficulty levels, sort by frequency of use or last performed
- Filters like "equipment type" help users find exercises quickly

**Metadata Display:**
- Last performed date is universal
- Total sessions/volume over time shows consistency
- Personal records highlighted with badges or color coding

**Search/Filter:**
- Instant text search by exercise name (not category browsing)
- Multiple simultaneous filters (muscle group + equipment + difficulty) for advanced users
- "Recently used" or "most frequent" as default sort helps discovery

## Sources

**Ecosystem & UX Patterns:**
- [JEFIT Best Workout Apps 2026](https://www.jefit.com/wp/guide/best-workout-apps-for-2026-top-options-tested-and-reviewed-by-pro/)
- [Fitness App UI Design Principles](https://stormotion.io/blog/fitness-app-ux/)
- [Best Apps for Tracking Exercise Goals](https://strive-workout.com/2026/02/02/best-apps-for-tracking-exercise-goals/)
- [Best Workout Tracker App 2026](https://www.hevyapp.com/best-workout-tracker-app/)
- [Fitness App UX Best Practices](https://www.zfort.com/blog/How-to-Design-a-Fitness-App-UX-UI-Best-Practices-for-Engagement-and-Retention)

**Database & Schema Design:**
- [Database Schema for Fitness Apps](https://www.back4app.com/tutorials/how-to-build-a-database-schema-for-a-fitness-tracking-application)
- [Database Design for Health Tracking](https://www.geeksforgeeks.org/dbms/how-to-design-a-database-for-health-and-fitness-tracking-applications/)
- [Fitness Application Development Guide](https://mobidev.biz/blog/fitness-application-development-guide-best-practices-and-case-studies)

**Exercise Library Features:**
- [Best Strength Training Apps 2026](https://www.jefit.com/wp/guide/best-strength-training-apps-for-2026-7-options-tested-by-lifters/)
- [Fitbod App](https://apps.apple.com/us/app/fitbod-gym-fitness-planner/id1041517543)
- [JEFIT Exercise Database](https://www.jefit.com/exercises)
- [Muscle & Motion Strength Training](https://www.muscleandmotion.com/strength-training-app/)

**Exercise Management Patterns:**
- [Edit/Delete Workout - MapMyFitness](https://support.mapmyfitness.com/hc/en-us/articles/1500009118702-Edit-or-Delete-a-Workout)
- [MyFitnessPal Exercise Editing](https://support.myfitnesspal.com/hc/en-us/articles/360032272552-How-do-I-edit-or-delete-an-exercise-I-created)
- [Fitbod Editing Workouts](https://fitbod.zendesk.com/hc/en-us/articles/360006335593-Editing-Workouts)

**Progressive Overload & Tracking:**
- [Progressive Overload Guide - Hevy](https://www.hevyapp.com/progressive-overload/)
- [Best Progressive Overload Apps 2026](https://www.jefit.com/wp/guide/best-progressive-overload-apps-for-beginners-in-2026-top-5-reviewed-and-compared/)
- [Setgraph Progressive Overload Tracker](https://setgraph.app/articles/setgraph-the-best-workout-tracker-app-for-strength-training-and-progressive-overload)

**Common Mistakes & Pitfalls:**
- [7 Things People Hate in Fitness Apps](https://www.ready4s.com/blog/7-things-people-hate-in-fitness-apps)
- [Best App for Tracking Workouts - Common Mistakes](https://setgraph.app/ai-blog/app-for-tracking-workouts-best-choices-2025)

**Volume & Statistics Tracking:**
- [Strong Workout Tracker](https://apps.apple.com/us/app/strong-workout-tracker-gym-log/id464254577)
- [Hevy Performance Tracking](https://www.hevyapp.com/features/gym-performance/)
- [FitNotes Progress Tracking](http://www.fitnotesapp.com/progress_tracking/)
- [RepCount Statistics](https://apps.apple.com/us/app/repcount-gym-workout-tracker/id594982044)

**Exercise Organization & Filtering:**
- [ExRx Exercise Directory](https://exrx.net/Lists/Directory)
- [Personal Trainer Exercise Filtering](https://www.fitsw.com/blog/personal-trainer-exercise-filtering/)
- [Exercise API Documentation](https://www.api-ninjas.com/api/exercises)
- [StrengthLog Exercise Directory](https://www.strengthlog.com/exercise-directory/)

---
*Feature research for: Exercise Library (Simple Strength Assistant v1.1)*
*Researched: 2026-03-01*
*Confidence: MEDIUM - based on WebSearch of current 2026 fitness app patterns, no official documentation for domain standards*
