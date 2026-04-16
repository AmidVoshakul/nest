//! Researcher Hand agent
//!
//! Autonomous background agent that continuously researches topics,
//! compiles reports, and runs in the background without user intervention.
//! All actions go through the permission system.

use nest_api::agent::AgentState;
use nest_api::message::Message;
use nest_runtime::AgentRuntime;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct ResearcherHand {
    id: String,
    state: AgentState,
    topics: Vec<String>,
    poll_interval: Duration,
}

impl ResearcherHand {
    /// Create a new Researcher Hand
    pub fn new(topics: Vec<String>) -> Self {
        Self {
            id: "researcher-hand".to_string(),
            state: AgentState::Stopped,
            topics,
            poll_interval: Duration::from_secs(3600), // Check every hour
        }
    }

    /// Run the researcher main loop
    pub async fn run(&mut self, runtime: &mut AgentRuntime) -> anyhow::Result<()> {
        self.state = AgentState::Running;
        
        loop {
            // Check if we should stop
            if self.state == AgentState::Stopped {
                break;
            }

            // Research each topic
            for topic in &self.topics {
                self.research_topic(runtime, topic).await?;
            }

            // Sleep until next check
            tokio::time::sleep(self.poll_interval).await;
        }

        Ok(())
    }

    /// Research a single topic
    async fn research_topic(&mut self, runtime: &mut AgentRuntime, topic: &str) -> anyhow::Result<()> {
        // Request permission to access the network
        let params = serde_json::json!({
            "query": format!("latest news about {}", topic),
            "language": "en"
        });

        match runtime.execute_tool(&self.id, "web_search", params).await {
            Ok(result) => {
                // Process search results
                let report = self.compile_report(topic, result);
                
                // Request permission to write report to disk
                let write_params = serde_json::json!({
                    "path": format!("./reports/{}.md", topic.replace(" ", "_")),
                    "content": report
                });
                
                let _ = runtime.execute_tool(&self.id, "filesystem_write_file", write_params).await;
            }
            Err(_) => {
                // Permission denied - will try again later
            }
        }

        Ok(())
    }

    /// Compile research results into a markdown report
    fn compile_report(&self, topic: &str, results: serde_json::Value) -> String {
        use std::time::SystemTime;
        
        format!("# Research Report: {}\n\nGenerated at: {:?}\n\n## Results\n\n{}",
            topic,
            SystemTime::now(),
            results
        )
    }

    /// Stop the researcher hand
    pub fn stop(&mut self) {
        self.state = AgentState::Stopped;
    }
}
