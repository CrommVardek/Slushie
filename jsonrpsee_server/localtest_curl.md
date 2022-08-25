
## Tests:

http address generated when server start "http://127.0.0.1:51423" where -> 51423 - port

- `nullifierHash` - string of 32 symbols
- `fee` - any value
- `recipient` - any receiver address
- `root` - value that generated in contract

Call correct curl:
```bash
curl -X POST -H 'Content-Type: application/json' -d '{"jsonrpc":"2.0","id":1,"method":"withdraw","params":["vpydjyqtbryvuflbjpcuzjtbbthfjymc","0x4ce946e968a0b477960eef24aafe0997350ba8f168ba2e4a546773556bdd1458", "10", "5GcSQPCVXrrWDjPXNnajYDqq24qa92V98cSW9xMzosDDnF3u"]}' http://127.0.0.1:57823
```

Call Error "Invalid nullifierHash":
```bash
curl -X POST -H 'Content-Type: application/json' -d '{"jsonrpc":"2.0","id":1,"method":"withdraw","params":["0x5GcSQPCVXrrWDjPXNnajYDqq24qa92V98cSW9xMzosDDnF","0x4ce946e968a0b477960eef24aafe0997350ba8f168ba2e4a546773556bdd1458", "10", "5GcSQPCVXrrWDjPXNnajYDqq24qa92V98cSW9xMzosDDnF3u"]}' http://127.0.0.1:57823
```

Call Error "Invalid root":
```bash
curl -X POST -H 'Content-Type: application/json' -d '{"jsonrpc":"2.0","id":1,"method":"withdraw","params":["vpydjyqtbryvuflbjpcuzjtbbthfjymc","0x4", "10", "5GcSQPCVXrrWDjPXNnajYDqq24qa92V98cSW9xMzosDDnF3u"]}' http://127.0.0.1:57823 
```

Call Error "Invalid fee":
```bash
curl -X POST -H 'Content-Type: application/json' -d '{"jsonrpc":"2.0","id":1,"method":"withdraw","params":["vpydjyqtbryvuflbjpcuzjtbbthfjymc","0x4ce946e968a0b477960eef24aafe0997350ba8f168ba2e4a546773556bdd1458", "xx", "5GcSQPCVXrrWDjPXNnajYDqq24qa92V98cSW9xMzosDDnF3u"]}' http://127.0.0.1:57823
```