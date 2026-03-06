# Quick Railway Bootstrap Setup

Railway provides a public IP that both your laptop and Codespaces can connect to.

## Steps

### 1. Deploy to Railway

```bash
# In your repo root
railway login
railway init
railway up
```

### 2. Get Railway Node Info

Once deployed, Railway will show logs with:
```
📍 Local peer id: 12D3KooW...
🌐 Listening on /ip4/0.0.0.0/tcp/8333
```

### 3. Get Railway Public Domain

Railway gives you a domain like: `your-app.railway.app`

But we need the IP. Railway will expose port 8333 automatically.

### 4. Connect From Anywhere

**From your laptop:**
```bash
./target/release/mrbn-node --data-dir ./data_test --port 8334 --bootstrap "/dns4/your-app.railway.app/tcp/8333/p2p/12D3KooW..."
```

**From Codespaces:**
```bash
./target/release/mrbn-node --data-dir ./data_test --port 8334 --bootstrap "/dns4/your-app.railway.app/tcp/8333/p2p/12D3KooW..."
```

## Why This Works

- Railway: Public IP, accepts incoming connections ✅
- Your Laptop: Behind NAT, but can make outgoing connections ✅
- Codespaces: Behind NAT, but can make outgoing connections ✅

Both your laptop and Codespaces can connect TO Railway, even though they can't connect to each other directly.
