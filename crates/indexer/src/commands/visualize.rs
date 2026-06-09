use geometry::visualize::MapVisualizer;

use crate::cli::VisualizeArgs;

pub async fn visualize_command(args: VisualizeArgs) -> anyhow::Result<()> {
    let mut visualizer = MapVisualizer::new();

    Ok(())
}
