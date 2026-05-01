use crate::models::ExerciseMetadata;

#[derive(Clone, Debug, PartialEq)]
pub struct PlanExercise {
    pub id: String,
    pub exercise: ExerciseMetadata,
    pub planned_sets: u32,
    pub position: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WorkoutPlan {
    pub id: String,
    pub started_at: Option<f64>,
    pub ended_at: Option<f64>,
    pub exercises: Vec<PlanExercise>,
}
