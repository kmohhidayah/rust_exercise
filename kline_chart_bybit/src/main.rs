use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use futures_util::{SinkExt, StreamExt};
use kline_chart_bybit::{
    constants::{USER_AGENT, VISIBLE_RANGE, WEBSOCKET_URL},
    models::{KlineResponse, SubscribeMessage},
    ui::CandlestickChart,
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{io, time::Duration};
use tokio::time::sleep;
use tokio_tungstenite::{
    connect_async,
    tungstenite::{client::IntoClientRequest, protocol::Message},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // WebSocket setup
    let mut request = WEBSOCKET_URL.into_client_request()?;
    request
        .headers_mut()
        .insert("User-Agent", USER_AGENT.parse()?);

    let (ws_stream, _) = connect_async(request).await?;
    println!("WebSocket connected");

    let (mut write, mut read) = ws_stream.split();

    // Subscribe to ETHUSDT kline
    let subscribe_msg = SubscribeMessage {
        op: "subscribe".to_string(),
        args: vec!["kline.1.ETHUSDT".to_string()],
    };

    write
        .send(Message::Text(serde_json::to_string(&subscribe_msg)?))
        .await?;

    // Terminal setup
    let terminal = setup_terminal()?;
    let mut chart = CandlestickChart::new(VISIBLE_RANGE);

    // Main event loop
    run_event_loop(&mut chart, &mut read, terminal).await?;

    // Cleanup
    cleanup_terminal()?;

    Ok(())
}

fn setup_terminal() -> io::Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    Terminal::new(CrosstermBackend::new(stdout))
}

fn cleanup_terminal() -> io::Result<()> {
    disable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, LeaveAlternateScreen, DisableMouseCapture)?;
    Ok(())
}

async fn run_event_loop(
    chart: &mut CandlestickChart,
    read: &mut futures_util::stream::SplitStream<
        tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
    >,
    mut terminal: Terminal<CrosstermBackend<io::Stdout>>,
) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        tokio::select! {
            msg = read.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        if let Ok(response) = serde_json::from_str::<KlineResponse>(&text) {
                            for kline_data in response.data {
                                chart.update_from_kline(&kline_data);
                            }
                        }
                    }
                    Some(Err(e)) => {
                        eprintln!("WebSocket error: {}", e);
                        break;
                    }
                    None => break,
                    _ => {}
                }
            }

            _ = sleep(Duration::from_millis(100)) => {
                if check_quit()? {
                    break;
                }

                terminal.draw(|f| {
                    chart.draw(f, f.size());
                })?;
            }
        }
    }
    Ok(())
}

fn check_quit() -> io::Result<bool> {
    if event::poll(Duration::from_millis(0))? {
        if let Event::Key(key) = event::read()? {
            if key.code == KeyCode::Char('q') {
                return Ok(true);
            }
        }
    }
    Ok(false)
}
