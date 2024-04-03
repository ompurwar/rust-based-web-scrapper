use fantoccini::{error::NewSessionError, Client, Locator};
use serde_json::Value;
use std::time::{Duration, Instant};
use tokio::time::sleep;

// Constants for configuration
// const WEB_LINK: &str = "https://www.nykaa.com/fragrance/men/perfumes-edt-edp/c/974";
const WEB_LINK: &str =
    "https://www.nykaa.com/bath-body/feminine-hygiene/sanitary-napkins/c/391?root=nav_4";
// const WEB_LINK: &str = "https://www.nykaafashion.com/men/footwear/c/6857";
const PRODUCT_TEXT_CLASS: &str = "css-xrzmfa";
const PRODUCT_LINK_CLASS: &str = "css-qlopj4";
const TIMEOUT_SECONDS: u64 = 30; // Timeout for page load wait

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = setup_client().await?;
    client.goto(WEB_LINK).await?;

    wait_for_page_load(&client).await?;

    // Extract and print the data after the page is fully loaded
    match extract_and_print_data(&client).await {
        Ok(_) => println!("Data extracted successfully."),
        Err(e) => {
            // Log the error and continue with the program, or perform some recovery action
            println!("Failed to extract data: {}", e);
            // Consider retry logic, or proceed without the data
            client.clone().close().await?;
        }
    }
    client.clone().close().await?;
    Ok(())
}

async fn setup_client() -> Result<Client, NewSessionError> {
    let proxy: Value = serde_json::json!({
        "proxyType": "manual",
        "httpProxy": "50.174.145.8",
        "sslProxy": "80",
    });
    let caps = serde_json::json!({
        "browserName":"chrome",
        "proxy": proxy,
    });

    let caps_map = caps
        .as_object()
        .cloned()
        .expect("Caps must be a JSON object");
    fantoccini::ClientBuilder::native()
        .capabilities(caps_map)
        .connect("http://localhost:4444")
        .await
}

async fn wait_for_page_load(client: &Client) -> Result<(), Box<dyn std::error::Error>> {
    let start = Instant::now();
    let timeout = Duration::from_secs(TIMEOUT_SECONDS);
    let js_check = "return document.readyState;";

    while Instant::now().duration_since(start) < timeout {
        if let Some(ready_state) = client.execute(js_check, vec![]).await?.as_str() {
            if ready_state == "complete" {
                println!("Page has loaded.");
                return Ok(());
            }
        }
        sleep(Duration::from_secs(1)).await;
    }

    Err("Timeout waiting for page load".into())
}

async fn extract_and_print_data(client: &Client) -> Result<(), Box<dyn std::error::Error>> {
    let current_url = client.current_url().await?;
    println!("Current URL: {}\n", current_url);

    // Use find_all to get all elements matching the CSS selector
    let product_containers = client.find_all(Locator::Css(".css-d5z3ro")).await?;

    for (index, container) in product_containers.iter().enumerate() {
        let product_info = container
            .find(Locator::Css(&format!(".{}", PRODUCT_TEXT_CLASS)))
            .await?
            .html(true)
            .await?;
        let link = container
            .find(Locator::Css(format!(".{}", PRODUCT_LINK_CLASS).as_str()))
            .await?
            .attr("href")
            .await?;
        let image_link = container
            .find(Locator::Css(".css-11gn9r6"))
            .await?
            .attr("src")
            .await?
            .unwrap_or("no src for image.".to_string());
        let price_tag = container.find(Locator::Css(".css-17x46n5")).await?;
        let mrp = price_tag
            .find(Locator::Css("span>span"))
            .await?
            .html(true)
            .await?;
        let price_tag_container = container.find(Locator::Css(".css-1d0jf8e")).await?;
        let discounted_price = price_tag_container
            .find(Locator::Css(".css-111z9ua"))
            .await?
            .html(true)
            .await?;
        let discounted_rate = price_tag_container
            .find(Locator::Css(".css-cjd9an"))
            .await?
            .html(true)
            .await?;

        println!(
            "Product Container {}: \n{}\n================\n{}\n{}\n{}\ndiscount price:{} \ndiscount rate:{}",
            index + 1,
            product_info,
            link.unwrap_or("no link found.".to_string()),
            image_link,
            mrp,
            discounted_price,
            discounted_rate,
        );
        // Add additional logic here as needed, for example, parsing or saving the data
    }

    Ok(())
}
