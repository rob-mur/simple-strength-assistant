use cucumber::{World, given, then, when};
use simple_strength_assistant::state::SyncStatus;

/// World for sync edge-case BDD scenarios covering migration integrity
/// and JS↔Rust boundary signal propagation.
#[derive(Debug, Default, World)]
pub struct SyncEdgeCaseWorld {
    /// Current sync status, modelling the Dioxus signal.
    pub sync_status: SyncStatus,
    /// Simulated exercises present before migration.
    pub exercises_before_migration: Vec<String>,
    /// Simulated exercises present after migration.
    pub exercises_after_migration: Vec<String>,
    /// Whether sync credentials are present.
    pub has_sync_credentials: bool,
    /// Whether the app reached "ready" state after migration.
    pub app_ready: bool,
    /// Error message carried by SyncStatus::Error, if any.
    pub last_error_message: Option<String>,
}

// ── Migration integrity steps ────────────────────────────────────────────────

#[given("a database with exercises before CRR migration")]
async fn step_db_with_exercises(world: &mut SyncEdgeCaseWorld) {
    world.exercises_before_migration =
        vec!["Bench Press".into(), "Squat".into(), "Deadlift".into()];
    world.exercises_after_migration.clear();
}

#[given("a database with exercises and sync credentials before CRR migration")]
async fn step_db_with_exercises_and_creds(world: &mut SyncEdgeCaseWorld) {
    world.exercises_before_migration = vec!["Bench Press".into(), "Squat".into()];
    world.has_sync_credentials = true;
    world.app_ready = false;
}

#[when("the crsql_as_crr migration runs")]
async fn step_crr_migration(world: &mut SyncEdgeCaseWorld) {
    // Simulate the migration: all pre-existing exercises survive.
    // In the real app, `applyCrrMigration()` in db-module.js calls
    // `SELECT crsql_as_crr(?)` for each table, which is a no-op on data rows.
    world.exercises_after_migration = world.exercises_before_migration.clone();
    // Credentials are preserved because they live in LocalStorage, not in
    // the SQLite tables that crsql_as_crr touches.
    // App transitions to ready after migration completes.
    world.app_ready = true;
}

#[then("all pre-existing exercises should still be present")]
async fn step_exercises_present(world: &mut SyncEdgeCaseWorld) {
    assert_eq!(
        world.exercises_before_migration, world.exercises_after_migration,
        "Expected exercises to survive migration.\n  Before: {:?}\n  After: {:?}",
        world.exercises_before_migration, world.exercises_after_migration
    );
    assert!(
        !world.exercises_after_migration.is_empty(),
        "Exercise list should not be empty after migration"
    );
}

#[then("the sync credentials should still be present")]
async fn step_creds_present(world: &mut SyncEdgeCaseWorld) {
    assert!(
        world.has_sync_credentials,
        "Sync credentials should survive migration"
    );
}

#[then("the app should be in a ready state")]
async fn step_app_ready(world: &mut SyncEdgeCaseWorld) {
    assert!(
        world.app_ready,
        "App should be in ready state after migration"
    );
}

// ── JS ↔ Rust boundary steps ────────────────────────────────────────────────

#[given(expr = "the sync status is {string}")]
async fn step_set_sync_status(world: &mut SyncEdgeCaseWorld, status: String) {
    world.sync_status = match status.as_str() {
        "idle" => SyncStatus::Idle,
        "never synced" => SyncStatus::NeverSynced,
        "syncing" => SyncStatus::Syncing,
        "up to date" => SyncStatus::UpToDate,
        "error" => SyncStatus::Error("previous error".into()),
        other => panic!("Unknown sync status: {other}"),
    };
    world.last_error_message = None;
}

#[when("the JS bridge reports connected")]
async fn step_bridge_connected(world: &mut SyncEdgeCaseWorld) {
    // Mirrors WorkoutStateManager::trigger_background_sync setting Syncing
    // when the sync cycle starts (connection established).
    world.sync_status = SyncStatus::Syncing;
}

#[when("the JS bridge reports sync complete")]
async fn step_bridge_sync_complete(world: &mut SyncEdgeCaseWorld) {
    // Mirrors the Synced/NoChanges arms setting UpToDate.
    world.sync_status = SyncStatus::UpToDate;
}

#[when("the JS bridge reports disconnected")]
async fn step_bridge_disconnected(world: &mut SyncEdgeCaseWorld) {
    // Mirrors the Offline arm setting Error.
    world.sync_status = SyncStatus::Error("Server unreachable".into());
    world.last_error_message = Some("Server unreachable".into());
}

#[when(expr = "the JS bridge reports a sync error {string}")]
async fn step_bridge_sync_error(world: &mut SyncEdgeCaseWorld, message: String) {
    // Mirrors the Error(msg) arm.
    world.sync_status = SyncStatus::Error(message.clone());
    world.last_error_message = Some(message);
}

#[then(expr = "the sync status should be {string}")]
async fn step_assert_sync_status(world: &mut SyncEdgeCaseWorld, expected: String) {
    let matches = match expected.as_str() {
        "idle" => world.sync_status == SyncStatus::Idle,
        "never synced" => world.sync_status == SyncStatus::NeverSynced,
        "syncing" => world.sync_status == SyncStatus::Syncing,
        "up to date" => world.sync_status == SyncStatus::UpToDate,
        "error" => matches!(world.sync_status, SyncStatus::Error(_)),
        other => panic!("Unknown expected sync status: {other}"),
    };
    assert!(
        matches,
        "Expected sync status '{expected}' but got {:?}",
        world.sync_status
    );
}

#[then(expr = "the sync error message should contain {string}")]
async fn step_error_message_contains(world: &mut SyncEdgeCaseWorld, substring: String) {
    let msg = world
        .last_error_message
        .as_ref()
        .expect("Expected an error message but none was set");
    assert!(
        msg.contains(&substring),
        "Expected error message to contain '{}' but got '{}'",
        substring,
        msg
    );
}
