
use std::env;
use tokio::runtime::Runtime;
use haikan::sqs::Messenger;

fn main() {
    // In order for this test to work, you need the appropriate credentials in ~/.aws,
    // The associated IAM role needs to have permission to access the SQS Queue,
    // and you need to specify the SQS_URL
    let sqs_url = env::var("SQS_URL").unwrap();
    println!("SQS URL={}", &sqs_url);
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let messenger = Messenger::new("us-east-1", &sqs_url).await;
        let messages = messenger.poll_strings(false).await.unwrap();
        for message in messages {
            println!("GOT MESSAGE {}", &message);
        }
     })
}


