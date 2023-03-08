use mobc_redis;
use mobc_redis::redis::AsyncCommands; // bring this trait into scope 
use rand::{Rng, distributions::Alphanumeric}; 
use tokio::runtime::Runtime;
use serde::{Serialize, Deserialize};
use haikan::redis::*;

// use different keys for different tests
// remember they all get executed at once asynchronously
const OBSCURE_TEST_KEY_1: &'static str = "_OBSCURE_TEST_KEY_1";
const OBSCURE_TEST_KEY_2: &'static str = "_OBSCURE_TEST_KEY_2";


fn gen_rand_int() -> i32 {
    rand::thread_rng().gen_range(1..1000)
}

// This is a struct you will serialize to and from Redis 
#[derive(Serialize, Deserialize)]
struct DemoStruct {
    id: i32,
    name: String,
}

impl mobc_redis::redis::ToRedisArgs for DemoStruct {
    fn write_redis_args<W>(&self, out: &mut W)
        where
            W: ?Sized + mobc_redis::redis::RedisWrite {
        out.write_arg_fmt(serde_json::to_string(self).expect("Can't serialize DemoStruct"))
    }
}


fn main() {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        
        // establish a connection pool 
        let rpool = new_pool_from_env().await.unwrap();
        
        
        // set a key to a random integer and ensure you can get it back 
        let mut rconn = rpool.get().await.unwrap();
        let rand_int = gen_rand_int();
        let _ : () = rconn.set(OBSCURE_TEST_KEY_1, rand_int).await.unwrap();
        let ox: Option<i32> = rconn.get(OBSCURE_TEST_KEY_1).await.unwrap();
        assert_eq!(ox.unwrap(), rand_int);
        println!("redis::get_set_int passed: {} == {}", ox.unwrap(), rand_int);
        

        // ensure you get delete a key and then get the None variant back 
        let _x = rediserde::del(&rpool, OBSCURE_TEST_KEY_2).await.unwrap();
        let ods2: Option<DemoStruct> = rediserde::get(&rpool, OBSCURE_TEST_KEY_2).await.unwrap();
        assert!(ods2.is_none());
        println!("Verified the key was deleted!"); 


        // Save a struct to a key and then read it back again 
        let id = gen_rand_int();
        let name: String = rand::thread_rng().sample_iter(&Alphanumeric).take(7).map(char::from).collect();
        let ds = DemoStruct{id, name};
        let _x = rediserde::set(&rpool, OBSCURE_TEST_KEY_2, &ds).await.unwrap();
        let ods2: Option<DemoStruct> = rediserde::get(&rpool, OBSCURE_TEST_KEY_2).await.unwrap();
        let ds2 = ods2.unwrap();
        assert_eq!(&ds.id, &ds2.id);
        assert_eq!(&ds.name, &ds2.name);
        println!("Congrats! You saved a struct to Redis and read it back again");

    })
}


