use clap::Parser;
use mihomo_speedtest_rs::{
    cli::{Cli, progress::SpeedTestProgress},
    config::ConfigLoader,
    core::{MihomoRunner, RealSpeedTester, SpeedTester},
    output::{ConfigExporter, ResultFormatter},
};
use std::process;
use tracing::{error, info, warn};

#[tokio::main]
async fn main() {
    // Parse command line arguments
    let args = Cli::parse();

    // Handle info commands first
    if args.show_author {
        println!("{}", env!("CARGO_PKG_AUTHORS"));
        return;
    }

    if args.show_about {
        println!("{}", env!("CARGO_PKG_DESCRIPTION"));
        return;
    }
    // Ensure config is provided for normal operation
    let config_paths = match &args.config_paths {
        Some(paths) => paths.clone(),
        None => {
            eprintln!("Error: --config is required for normal operation");
            process::exit(1);
        }
    };

    // Initialize logging
    let log_level = if args.verbose {
        tracing::Level::DEBUG
    } else {
        tracing::Level::INFO
    };

    tracing_subscriber::fmt()
        .with_max_level(log_level)
        .with_target(false)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .init();

    if let Err(e) = run(args, &config_paths).await {
        error!("Application error: {}", e);
        eprintln!("Application error: {}", e);
        process::exit(1);
    }
}

async fn run(args: Cli, config_paths: &str) -> mihomo_speedtest_rs::Result<()> {
    info!("🚀 Starting Mihomo SpeedTest");

    // Display parameter table unless in JSON output mode
    if !args.json_output {
        let param_table = args.create_parameter_table();
        println!("\n📋 Configuration Parameters");
        println!("{}", param_table.format_table());
        println!(
            "📊 Summary: {}/{} parameters customized\n",
            param_table.customized_count(),
            param_table.total_count()
        );
    }

    // Load configuration
    let loader = ConfigLoader::new();
    let mut proxies = loader.load_from_paths(config_paths).await?;

    if proxies.is_empty() {
        warn!("No proxies loaded from configuration");
        return Ok(());
    }

    info!("📋 Loaded {} proxies", proxies.len());

    // Apply name filtering
    if args.filter_regex != ".+" {
        let regex = regex::Regex::new(&args.filter_regex)?;
        let original_count = proxies.len();
        proxies.retain(|p| regex.is_match(&p.name));
        info!(
            "🔍 Filtered by regex '{}': {} -> {} proxies",
            args.filter_regex,
            original_count,
            proxies.len()
        );
    }

    // Apply keyword blocking
    if let Some(ref keywords) = args.block_keywords {
        let block_list: Vec<String> = keywords
            .split('|')
            .map(|s| s.trim().to_lowercase())
            .filter(|s| !s.is_empty())
            .collect();

        if !block_list.is_empty() {
            let original_count = proxies.len();
            proxies.retain(|p| {
                !block_list
                    .iter()
                    .any(|keyword| p.name.to_lowercase().contains(keyword))
            });
            info!(
                "🚫 Blocked keywords: {} -> {} proxies",
                original_count,
                proxies.len()
            );
        }
    }

    if proxies.is_empty() {
        warn!("No proxies remaining after filtering");
        return Ok(());
    }

    // Create speed tester
    let config = args.to_speedtest_config();

    // Test proxies
    info!("🧪 Starting speed tests for {} proxies", proxies.len());
    let results = if args.use_mihomo {
        // Use mihomo for real proxy testing
        info!("🔧 Using mihomo process for real proxy testing");

        let mihomo_runner = MihomoRunner::new(
            &args.mihomo_config_dir,
            args.mihomo_binary.as_ref(),
            args.mihomo_api_port,
            args.mihomo_proxy_port,
        )?;

        let mut real_tester = RealSpeedTester::new(mihomo_runner, config);
        let progress = SpeedTestProgress::new(proxies.len() as u64);
        let results = real_tester.test_proxies_with_progress(&proxies, Some(&progress)).await?;
        progress.finish_with_message("Speed tests completed!");
        results
    } else {
        // Use original direct testing method
        let tester = SpeedTester::new(config);

        if args.max_concurrent > 1 {
            let progress = SpeedTestProgress::new(proxies.len() as u64);
            let results = tester
                .test_proxies_concurrent(proxies.clone(), args.max_concurrent)
                .await?;
            progress.finish_with_message("Speed tests completed!");
            results
        } else {
            let progress = SpeedTestProgress::new(proxies.len() as u64);
            let progress_clone = progress.clone();
            let results = tester
                .test_proxies(
                    proxies.clone(),
                    Some(Box::new(move |result| {
                        progress_clone.update(result);
                    })),
                )
                .await?;
            progress.finish_with_message("Speed tests completed!");
            results
        }
    };

    // Filter results based on performance criteria
    let filtered_results: Vec<_> = results
        .into_iter()
        .filter(|result| {
            // Primary success criteria: has latency and latency is within acceptable range
            if let Some(latency) = result.latency {
                if latency <= args.max_latency {
                    // Additional criteria for non-fast mode
                    if !args.fast_mode {
                        // Check download speed (convert MB/s to bytes/s)
                        let min_download_bytes = args.min_download_speed * 1024.0 * 1024.0;
                        if result.download_speed < min_download_bytes {
                            return false;
                        }

                        // Check upload speed (convert MB/s to bytes/s)
                        let min_upload_bytes = args.min_upload_speed * 1024.0 * 1024.0;
                        if result.upload_speed < min_upload_bytes {
                            return false;
                        }
                    }
                    return true;
                }
            }

            false
        })
        .collect();

    info!(
        "✅ {} proxies passed performance criteria",
        filtered_results.len()
    );

    // Format and display results
    let formatter = ResultFormatter::new(args.json_output, !args.json_output);
    let output = formatter.format_results(&filtered_results);
    println!("{output}");

    if !args.json_output {
        println!("{}", formatter.format_summary(&filtered_results));
    }

    // Export results if requested
    if let Some(ref output_path) = args.output {
        info!("💾 Exporting results to: {}", output_path);

        if args.rename_nodes {
            let renamed_proxies =
                ConfigExporter::rename_proxies_with_stats( &filtered_results);
            ConfigExporter::export_clash_config(&renamed_proxies, output_path)
                .await?;
        } else {
            ConfigExporter::export_clash_config(&filtered_results, output_path).await?;
        }

        info!("✅ Export completed");
    }

    info!("🎉 All tasks completed successfully!");
    Ok(())
}
