use {
    futures_lite::stream::StreamExt,
    lapin::{
        options::*, types::FieldTable, BasicProperties, Channel, Connection, ConnectionProperties,
        Result,
    },
    serde::{Deserialize, Serialize},
    serde_json::to_string,
    tokio::task::{spawn, JoinHandle},
};

const CHANNEL_NAME: &'static str = "rmq_channel";

/// This is the message of communication between the backend and frontend.
#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum EditorChange {
    Insertion {
        character: char,
        line: usize,
        column: usize,
    },
    Deletion {
        line: usize,
        column: usize,
    },
}

/// This is the entrypoint for connecting to the desired frontend.
/// More specifically, this forms a connection to RabbitMQ, which will broker messages to the frontend.
pub struct TransferInterface {
    consumer_channel: Option<Channel>,
    producer_channel: Option<Channel>,
    handle: Option<JoinHandle<()>>,
}

impl TransferInterface {
    /// This is a message queuing middleware for handling connections between the frontend and backend.
    pub fn new() -> Self {
        Ok(Self {
            handle: None,
            consumer_channel: None,
            producer_channel: None,
        })
    }

    pub async fn init(&mut self) -> Result<()> {
        let addr =
            std::env::var("AMQP_ADDR").unwrap_or_else(|_| "amqp://127.0.0.1:5672/%2f".into());
        let connection = Connection::connect(&addr, ConnectionProperties::default())
            .await
            .expect("Failed to set up connection to RabbitMQ server.");
        let consumer_channel = connection
            .create_channel()
            .await
            .expect("Consumer channel failed to initialize");
        let producer_channel = connection
            .create_channel()
            .await
            .expect("Producer channel failed to initialize.");
        let producer_queue = producer_channel
            .queue_declare(
                "hello",
                QueueDeclareOptions::default(),
                FieldTable::default(),
            )
            .await
            .expect("Producer queue failed to initialize.");
        let mut consumer_queue = consumer_channel
            .basic_consume(
                "hello",
                "my_consumer",
                BasicConsumeOptions::default(),
                FieldTable::default(),
            )
            .await
            .expect("Consumer queue failed to initialize.");

        self.consumer_channel = Some(consumer_channel);
        self.producer_channel = Some(producer_channel);

        self.handle = Some(spawn(async move {
            while let Some(delivery) = consumer_queue.next().await {
                let delivery = delivery.expect("error in consumer");
                delivery.ack(BasicAckOptions::default()).await.expect("ack");
            }
        }));

        Ok(())
    }

    /// Sends the character update to the message queue.
    /// `rk`: the routing key for the published message.
    /// `character`: the character that is being inserted or deleted.
    /// `line`: the line number of the updated character.
    /// `col`: the column number of the updated character.
    pub async fn send(
        &self,
        rk: &str,
        line: usize,
        column: usize,
        character: Option<char>,
    ) -> Result<()> {
        let msg = match character {
            Some(character) => EditorChange::Insertion {
                character,
                line,
                column,
            },
            None => EditorChange::Deletion { line, column },
        };

        match &self.producer_channel {
            Some(channel) => {
                let payload = to_string(&msg).expect("Unable to serialize character update.");
                let confirmation = channel
                    .basic_publish(
                        "",
                        "hello",
                        BasicPublishOptions::default(),
                        payload.as_bytes().to_vec(),
                        BasicProperties::default(),
                    )
                    .await?
                    .await?;
            }
            None => panic!("Attempted to publish to channel, but channel was not initialized."),
        };

        Ok(())
    }

    /// Gets data from the message queue.
    pub async fn recv(&mut self) -> Result<()> {
        Ok(())
    }
}
