Create Websocket connection with: 
```
let ws = new WebSocket("http://127.0.0.1:9000")
```

Subscribe for trades with: 
```
let payload = {
    method: "SUBSCRIBE",
    event: "TRADE",
    symbol: "SOL_USDT"
};

ws.send(JSON.stringify(payload));
```

Listen for messages with:
```
ws.onmessage = function(event) {
    console.log('Received:', event.data);
};
```

Unsubscribe for trades with:
```
let payload = {
    method: "UNSUBSCRIBE",
    event: "TRADE",
    symbol: "SOL_USDT"
};

ws.send(JSON.stringify(payload));
```