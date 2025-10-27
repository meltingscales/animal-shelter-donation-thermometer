use askama::Template;
use crate::ThermometerConfig;

#[derive(Template)]
#[template(path = "thermometer.svg")]
struct ThermometerTemplate {
    width: u32,
    height: u32,
    title_x: String,
    title_y: String,
    title_font_size: String,
    tube_x: String,
    tube_y: String,
    tube_width: String,
    tube_height: String,
    fill_x: String,
    fill_y: String,
    fill_width: String,
    fill_height: String,
    bulb_center_x: String,
    bulb_center_y: String,
    bulb_radius: String,
    bulb_fill_radius: String,
    percentage_markers: Vec<PercentageMarker>,
    text_x: String,
    achieved_y: String,
    achieved_amount: String,
    achieved_label_y: String,
    goal_y: String,
    goal_amount: String,
    goal_label_y: String,
    percent_y: String,
    progress_percent: String,
    percent_label_y: String,
    amount_font_size: String,
    label_font_size: String,
    percent_font_size: String,
    percent_label_font_size: String,
}

#[derive(Debug, Clone)]
struct PercentageMarker {
    line_x1: String,
    y: String,
    line_x2: String,
    text_x: String,
    text_y: String,
    font_size: String,
    percentage: i32,
}

/// Generate an SVG thermometer image based on the configuration
pub fn generate_thermometer_svg(config: &ThermometerConfig, width: u32) -> String {
    let total_raised: f64 = config.teams.iter().map(|t| t.total_raised).sum();
    let progress_percent = if config.goal > 0.0 {
        ((total_raised / config.goal) * 100.0).min(100.0)
    } else {
        0.0
    };

    // Calculate dimensions based on width
    let height = (width as f64 * 1.2) as u32; // Maintain aspect ratio
    let thermometer_width = width as f64 * 0.35;
    let thermometer_height = height as f64 * 0.6;
    let thermometer_x = width as f64 * 0.1;
    let thermometer_y = height as f64 * 0.15;

    // Thermometer dimensions
    let bulb_radius = thermometer_width * 0.4;
    let tube_width = thermometer_width * 0.35;
    let tube_height = thermometer_height - bulb_radius;
    let tube_x = thermometer_x + (thermometer_width - tube_width) / 2.0;
    let tube_y = thermometer_y;
    let bulb_center_x = thermometer_x + thermometer_width / 2.0;
    let bulb_center_y = tube_y + tube_height + bulb_radius;

    // Fill height based on progress
    let fill_height = (tube_height * progress_percent / 100.0).max(0.0);
    let fill_y = tube_y + tube_height - fill_height;

    // Text positioning
    let text_x = width as f64 * 0.55;
    let title_y = height as f64 * 0.1;
    let achieved_y = height as f64 * 0.35;
    let goal_y = height as f64 * 0.55;
    let percent_y = height as f64 * 0.75;

    // Generate percentage markers
    let percentages = [100, 80, 60, 40, 20, 0];
    let marker_length = thermometer_width * 0.25;
    let font_size = width as f64 * 0.02;

    let percentage_markers: Vec<PercentageMarker> = percentages
        .iter()
        .map(|&p| {
            let y = tube_y + tube_height * (1.0 - p as f64 / 100.0);
            let marker_x = tube_x - marker_length - 5.0;
            let text_x = marker_x - 5.0;

            PercentageMarker {
                line_x1: format!("{:.2}", marker_x),
                y: format!("{:.2}", y),
                line_x2: format!("{:.2}", tube_x - 5.0),
                text_x: format!("{:.2}", text_x),
                text_y: format!("{:.2}", y + font_size * 0.35),
                font_size: format!("{:.2}", font_size),
                percentage: p,
            }
        })
        .collect();

    let template = ThermometerTemplate {
        width,
        height,
        title_x: format!("{:.2}", width as f64 / 2.0),
        title_y: format!("{:.2}", title_y),
        title_font_size: format!("{:.2}", width as f64 * 0.035),
        tube_x: format!("{:.2}", tube_x),
        tube_y: format!("{:.2}", tube_y),
        tube_width: format!("{:.2}", tube_width),
        tube_height: format!("{:.2}", tube_height),
        fill_x: format!("{:.2}", tube_x + 2.5),
        fill_y: format!("{:.2}", fill_y),
        fill_width: format!("{:.2}", tube_width - 5.0),
        fill_height: format!("{:.2}", fill_height),
        bulb_center_x: format!("{:.2}", bulb_center_x),
        bulb_center_y: format!("{:.2}", bulb_center_y),
        bulb_radius: format!("{:.2}", bulb_radius),
        bulb_fill_radius: format!("{:.2}", bulb_radius - 3.0),
        percentage_markers,
        text_x: format!("{:.2}", text_x),
        achieved_y: format!("{:.2}", achieved_y),
        achieved_amount: format!("{:.2}", total_raised),
        achieved_label_y: format!("{:.2}", achieved_y + width as f64 * 0.03),
        goal_y: format!("{:.2}", goal_y),
        goal_amount: format!("{:.2}", config.goal),
        goal_label_y: format!("{:.2}", goal_y + width as f64 * 0.03),
        percent_y: format!("{:.2}", percent_y),
        progress_percent: format!("{:.0}", progress_percent),
        percent_label_y: format!("{:.2}", percent_y + width as f64 * 0.025),
        amount_font_size: format!("{:.2}", width as f64 * 0.06),
        label_font_size: format!("{:.2}", width as f64 * 0.025),
        percent_font_size: format!("{:.2}", width as f64 * 0.09),
        percent_label_font_size: format!("{:.2}", width as f64 * 0.022),
    };

    template.render().unwrap_or_else(|e| {
        eprintln!("Failed to render thermometer template: {}", e);
        String::from("<svg><text>Error rendering thermometer</text></svg>")
    })
}

/// Convert SVG to PNG with the specified scale
pub fn svg_to_png(svg_data: &str, scale: f32) -> Result<Vec<u8>, String> {
    use resvg::usvg;
    use tiny_skia::Pixmap;

    // Create a font database and load system fonts
    let mut fontdb = usvg::fontdb::Database::new();
    fontdb.load_system_fonts();

    // Parse the SVG with font database
    let mut opts = usvg::Options::default();
    opts.fontdb = std::sync::Arc::new(fontdb);

    let tree = usvg::Tree::from_str(svg_data, &opts)
        .map_err(|e| format!("Failed to parse SVG: {}", e))?;

    // Get the SVG size
    let size = tree.size();
    let width = (size.width() * scale) as u32;
    let height = (size.height() * scale) as u32;

    // Create a pixmap
    let mut pixmap = Pixmap::new(width, height)
        .ok_or_else(|| "Failed to create pixmap".to_string())?;

    // Render the SVG
    let transform = if scale != 1.0 {
        tiny_skia::Transform::from_scale(scale, scale)
    } else {
        tiny_skia::Transform::identity()
    };

    resvg::render(&tree, transform, &mut pixmap.as_mut());

    // Encode as PNG
    pixmap.encode_png()
        .map_err(|e| format!("Failed to encode PNG: {}", e))
}
