use crate::game::GameResult;
use burn::data::dataset::Dataset;

#[derive(Clone)]
struct GameDataset {
    dataset: Vec<GameResult>,
}

impl Dataset<GameResult> for GameDataset {
    fn get(&self, index: usize) -> Option<GameResult> {
        self.dataset.get(index).cloned()
    }

    fn len(&self) -> usize {
        self.dataset.len()
    }
}
