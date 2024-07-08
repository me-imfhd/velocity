## Key Features
## Performance Benchmarks
- **Order Placement:** <1ms
- **Processing Order & Publishing Events:** ~4ms (1-10ms)
- **Recieving Order Response** ~6ms-10ms
- **Persiting Orderbook Mutations Parallely:** ~10ms (5-40ms)
- **Database Updates:** ~25-40ms per trade (updating orders, updating balances, and inserting trades)

## Architecture
### Database
- **Scylla DB:** Velocity uses Scylla DB for low-latency database interactions, recovering orderbooks ensuring things never go wrong.
### In-Memory Storage
- **Orderbooks and User Balances:** The orderbooks for registered markets and user balances are stored directly in the engine's memory. This allows for quick access and updates.
- **Recovery Mechanism:** 
    - After validating the order and locking the in-memory user balance, the order is pushed into an MPSC channel, and continues processing the order and preparing the response. 
    - Another thread picks from the channel to insert the order and lock the user's balances in the database. 
    - If the engine goes down, ScyllaDB helps recovering the orderbook by replaying orders from the last 24 hours. User data is also reloaded from the database.
### Order Processing
- **Order Placement:** Orders are queued for the matching engine in under 1 millisecond. Each market has its own dedicated thread, allowing parallel handling of orders of different markets.
- **Order Validation and Execution:** After validation, orders are placed into a channel and picked up by a separate thread. This thread handles database entries and locks user balances. This is the secret of orderbook recovery mechanism of velocity exchange.
### Trade Matching
- **Balance Updates:** When an order is matched, the system exchanges traders' balances. Then the trades, ticker & depth and order updates are streamed and database is filled via a filler queue.
- **Database and Broadcasting:** The queue helps fill our database for long-term storage. We use WebSockets and pub/sub mechanisms to broadcast trades, tickers, and depth updates to subscribers and stream private order updates to the order maker from the matching engine directly before the queue.

<center><img src="./assets/architecture.png"></center>

## Videos
- **Architecture**  

https://github.com/me-imfhd/velocity/assets/114667178/6fb65dba-0aee-4536-af0c-2b5a1b5d2094

- **Performance Benchmarks**
  
https://github.com/me-imfhd/velocity/assets/114667178/976cf18e-29c4-47c6-aee2-12e6ac512e58

## Order Request Types:
- **Limit/Market** 
    - Market order are completely filled, lead by quote requests first, & is not stored. 
    - Completely filled limit orders are removed from orderbook.
- **Cancel/CancelAll** 
    - Cancelled orders are removed from orderbook.
- **OpenOrder/OpenOrders**
    - Consists of only partially filled limit orders.

## Api Collection
<center><img src="./assets/api_col.png"></center>

- **Check assets/api_collection.json for more**

## Remaining:
- Authentication
- Trading View
- Frontend Integration
