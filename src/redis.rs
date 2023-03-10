//! This module defines ergonomic ways to create and use a Redis connection pool.
//! The mobc crate is used to create an async pool. This was chosen for two reasons,
//! 1) familiar paralellism with the mobc-postgres crate used extensively elewhere
//! 2) [This page](https://blog.logrocket.com/using-redis-in-a-rust-web-service/) reports successful deployment in production using mobc:
//! 
//! The new_client_from_env() and new_pool_from_env() methods maket it easy to connect using these environment variables:
//! REDIS_HOST: The IP where the Redis server is running. Defauls to "127.0.0.1"
//! REDIS_PORT: The port on which the server is listening. Defaults to 6379
//! REDIS_PW: The authentication password for Redis
//! IS_TSL: If set to anything, rediss will be used instead of redis
//! 
//! Caching and getting structs is easy with the rediserde::get function:
//! you just need to implement impl mobc_redis::redis::ToRedisArgs
//! See examples/redis.rs for an example of how to do so. 

use std::{env, time::Duration};
use mobc::Pool;
use mobc_redis::{RedisConnectionManager, redis::{AsyncCommands, RedisResult, Client, aio::Connection}};
use crate::err::GenericError;

// constants for mobc redis connection pools
// see https://blog.logrocket.com/using-redis-in-a-rust-web-service/
const CACHE_POOL_MAX_OPEN: u64 = 16;


pub type RedisConn = Connection<RedisConnectionManager>;
pub type RedisPool = Pool<RedisConnectionManager>;


/// Return a new connection pool from the mobc_redis::Client struct
pub async fn new_pool_from_client(client: Client) -> Result<RedisPool, GenericError> {
    let manager = RedisConnectionManager::new(client);
    let pool = Pool::builder()
        //.get_timeout(Some(Duration::from_secs(CACHE_POOL_TIMEOUT_SECONDS)))
        .max_open(CACHE_POOL_MAX_OPEN)
        //.max_idle(CACHE_POOL_MAX_IDLE)
        //.max_lifetime(Some(Duration::from_secs(CACHE_POOL_EXPIRE_SECONDS)))
        //.max_lifetime(None)
        .build(manager);
    // try to connect now so you fail early
    let mut conn = pool.get().await?;
    Ok(pool)
}

/// Create a new pool from a client generated with these environment variables:
pub async fn new_pool_from_env() -> Result<RedisPool, GenericError> {
    let client = new_client_from_env()?;
    new_pool_from_client(client).await
}


/// Generate a new client based on a uri scheme, a host, and a password
pub fn new_client(uri_scheme: &str, redis_host: &str, redis_pw: &str) -> RedisResult<Client> {
    let redis_conn_url = format!("{}://:{}@{}", uri_scheme, redis_pw, redis_host);
    Client::open(redis_conn_url)
}

/// Generate a new client from environment variables
/// These are they key variables that must be set:
/// REDIS_HOST: Defaults to 127.0.0.1
/// REDIS_PORT: Defaults tos 6379
/// REDIS_PW: The password to authenticate
/// IS_TLS: uses rediss instead of redis
/// 
pub fn new_client_from_env() -> RedisResult<Client>  {
    let uri_scheme = match env::var("IS_TLS") {
        Ok(_) => "rediss",
        Err(_) => "redis",
    };

    let redis_host: String = match env::var("REDIS_HOST") {
        Ok(val) => val,
        Err(_) => {
            match env::var("REDIS_PORT")  {
                Ok(port) => format!("127.0.0.1:{}", port),
                Err(_) => "127.0.0.1:6379".to_string(),
            }
        },
    };
    let redis_pw: String = match env::var("REDIS_PW") {
        Ok(val) => val,
        Err(_) => "".to_string(),
    };
    new_client(&uri_scheme, &redis_host, &redis_pw)
}



pub mod rediserde {
    use super::{RedisPool};
    use mobc_redis::redis::AsyncCommands;
    use crate::err::GenericError;
    use serde::{Serialize, de::DeserializeOwned};
    use serde_json;


    /// Delete a key 
    pub async fn del(pool: &RedisPool, key: &str) -> Result<(), GenericError> {
        let mut rconn = pool.get().await?;
        let _ : () = rconn.del(key).await?;
        Ok(())
    }

    /// For a struct that can be deserialized,
    /// This helpful method gets a connection, gets the value stored at the key,
    /// deserializes it, and returns the desired struct
    pub async fn get<T: DeserializeOwned>(pool: &RedisPool, key: &str) -> Result<Option<T>, GenericError> {
        let mut rconn = pool.get().await?;
        let jz: String = match rconn.get(key).await {
            Ok(val) => val,
            Err(e) => {
                if e.to_string().contains("response was nil") {
                    return Ok(None)
                }
                return Err(e.into())
            }  
        };
        let t: T = serde_json::from_str(&jz)?;
        Ok(Some(t))
    }

    /// For a struct that can be serialized,
    /// This helpful method gets a connection, gets teh value stored at the key,
    /// deserializes it, and returns the desired struct 
    pub async fn set<T: Serialize>(pool: &RedisPool, key: &str, value: &T) -> Result<(), GenericError> {
        let mut rconn = pool.get().await?;
        let jz: String = serde_json::to_string(value)?;
        let _ : () = rconn.set(key, jz).await?;
        Ok(())
    }

    /// add a struct to a set
    pub async fn sadd<T: Serialize>(pool: &RedisPool, key: &str, value: &T) -> Result<(), GenericError> {
        let mut rconn = pool.get().await?;
        let jz: String = serde_json::to_string(value)?;
        let _ : () = rconn.sadd(key, jz).await?;
        Ok(())
    }

    /// add a string to a set
    pub async fn sadd_str(pool: &RedisPool, key: &str, val: &str) -> Result<(), GenericError> {
        let mut rconn = pool.get().await?;
        let _ : () = rconn.sadd(key, val).await?;
        Ok(())
    }

    pub async fn spop_str(pool: &RedisPool, key: &str) -> Result<Option<String>, GenericError> {
        // This pool.get() hangs sometimes with the error "Timed out in mobc". What to do?  
        let mut rconn = pool.get().await?;
        let jz: String = match rconn.spop(key).await {
            Ok(val) => val,
            Err(e) => {
                if e.to_string().contains("response was nil") {
                    return Ok(None)
                }
                return Err(e.into())
            }  
        };
        Ok(Some(jz))
    }


    pub async fn spop<T: DeserializeOwned>(pool: &RedisPool, key: &str) -> Result<Option<T>, GenericError> {
        let jz = match spop_str(pool, key).await? {
            Some(val) => val,
            None => return Ok(None),
        };
        let t: T = serde_json::from_str(&jz)?;
        Ok(Some(t))
    }

    pub async fn scard(pool: &RedisPool, key: &str) -> Result<usize, GenericError> {
        let mut rconn = pool.get().await?;
        let cardinality = rconn.scard(key).await?;
        Ok(cardinality)
    }

}





#[cfg(test)]
mod tests {
    use mobc_redis;
    use rand::{Rng, distributions::Alphanumeric}; 
    use tokio::runtime::Runtime;
    use serde::{Serialize, Deserialize};
    use super::*;

    // use different keys for different tests-
    // remember they all get executed at once asynchronously 
    const OBSCURE_TEST_KEY_1: &'static str = "_OBSCURE_TEST_KEY_1";
    const OBSCURE_TEST_KEY_2: &'static str = "_OBSCURE_TEST_KEY_2";

    fn gen_rand_int() -> i32 {
        rand::thread_rng().gen_range(1..1000)
    }

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

    #[test]
    fn get_set_int() {
        // ensure you can set and get a value 
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let rpool = new_pool_from_env().await.unwrap();
            let mut rconn = rpool.get().await.unwrap();
            let rand_int = gen_rand_int();
            let _ : () = rconn.set(OBSCURE_TEST_KEY_1, rand_int).await.unwrap();
            let ox: Option<i32> = rconn.get(OBSCURE_TEST_KEY_1).await.unwrap();
            assert_eq!(ox.unwrap(), rand_int);
            println!("redis::get_set_int passed: {} == {}", ox.unwrap(), rand_int);

        })
    }

    #[test]
    fn get_set_struct() {
        // ensure you save and load an instance of a struct 
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let rpool = new_pool_from_env().await.unwrap();
            // ensure you get delete a key and then get the None variant back 
            let _x = rediserde::del(&rpool, OBSCURE_TEST_KEY_2).await.unwrap();
            let ods2: Option<DemoStruct> = rediserde::get(&rpool, OBSCURE_TEST_KEY_2).await.unwrap();
            assert!(ods2.is_none());
            // Then set it and ensure you can get the Some() variant back
            let id = gen_rand_int();
            let name: String = rand::thread_rng().sample_iter(&Alphanumeric).take(7).map(char::from).collect();
            let ds = DemoStruct{id, name};
            let _x = rediserde::set(&rpool, OBSCURE_TEST_KEY_2, &ds).await.unwrap();
            let ods2: Option<DemoStruct> = rediserde::get(&rpool, OBSCURE_TEST_KEY_2).await.unwrap();
            let ds2 = ods2.unwrap();
            assert_eq!(&ds.id, &ds2.id);
            assert_eq!(&ds.name, &ds2.name);
        })
    }
}
