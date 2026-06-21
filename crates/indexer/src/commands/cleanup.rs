use std::collections::HashMap;

use colored::Colorize;

use crate::{cli::CleanupArgs, db::IndexDatabase};

pub async fn cleanup_command(args: CleanupArgs) -> anyhow::Result<()> {
    println!(
        "{}",
        "Connecting to database and getting all points...".blue()
    );

    let client = IndexDatabase::new().await?;
    let all_points = client.get_all_points().await?;

    println!(
        "{}",
        "Checking if there's any points with less than 4 images...".blue()
    );

    let mut angles: HashMap<(i64, i64), u32> = HashMap::new();

    for point in all_points.iter() {
        let key = (float_to_int(point.ilon), float_to_int(point.ilat));
        if let Some(existing) = angles.get_mut(&key) {
            *existing += 1;
        } else {
            angles.insert(key, 1);
        }
    }

    let total = angles.values().filter(|v| (**v) < 4).sum::<u32>();

    println!(
        "There are {} points which have less than 4 images.{}",
        total.to_string().blue(),
        if args.clean {
            ""
        } else {
            " Run the command with --clean to remove them."
        }
    );

    if args.clean {
        // TODO: Actually perform the cleanup
    }

    Ok(())
}

fn float_to_int(f: f64) -> i64 {
    (f * 1_000_000.0) as i64
}
