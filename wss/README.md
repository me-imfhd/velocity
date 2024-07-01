## Websocket streams

### Create Websocket connection
```
let ws = new WebSocket("http://127.0.0.1:9000")
```

### Subscribe to trades, ticker and depth streams
```

let payload = {
    method: "SUBSCRIBE",
    event: EVENT, // here event is "TRADE", "TICKER", "DEPTH"
    symbol: "SOL_USDT"
};

ws.send(JSON.stringify(payload));
```

### Subscribe to live order updates
```
let payload = {
    user_id: your_user_id,
    method: "SUBSCRIBE",
    event: "ORDER_STATUS",
    symbol: "SOL_USDT"
};

ws.send(JSON.stringify(payload));
```

### Listen for messages
```
ws.onmessage = function(event) {
    console.log('Received:', JSON.parse(event.data));
};
```