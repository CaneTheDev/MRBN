# How MRBN P2P Actually Works (Like BitTorrent)

## The Simple Truth

MRBN uses **Kademlia DHT** - the same technology as BitTorrent. Here's how it works:

### First Time Setup (Bootstrap)
1. You need to connect to ONE peer (any peer)
2. That peer shares its DHT routing table with you
3. Now you know about many other peers

### After First Connection
1. Your node saves the DHT routing table to disk
2. Next time you start, you load saved peers
3. **No bootstrap needed anymore!**

### How Peers Find Each Other
```
You → Connect to Peer A
Peer A → "Here are 20 other peers I know"
You → Connect to some of those peers
Those peers → "Here are more peers"
...and so on
```

Within minutes, you know about hundreds of peers through DHT propagation.

## Why Railway Doesn't Work

Railway uses a **TCP proxy** that expects standard protocols (HTTP, PostgreSQL, etc.).

libp2p uses a **custom protocol negotiation** that Railway's proxy doesn't understand.

## Real World Deployment

### Option 1: Public Bootstrap Nodes (Recommended)
- Deploy 3-5 nodes on real VPS (Oracle Free Tier, DigitalOcean, etc.)
- These become "seed nodes" like BitTorrent trackers
- Users connect to any one seed node
- After that, DHT takes over - no seeds needed

### Option 2: Peer Exchange
- First user shares their peer ID with second user (Discord, email, etc.)
- Second user connects using `--bootstrap`
- Both nodes now in DHT
- Third user can connect to either one
- Network grows organically

### Option 3: Hardcoded Seed List
- Include 10-20 known peer addresses in the code
- Node tries each one until it connects
- Like how Bitcoin has hardcoded seed nodes

## The Key Insight

**You only need ONE successful connection to join the network.**

After that, DHT does everything:
- Finds more peers automatically
- Maintains routing table
- Handles peer churn (nodes going offline)
- No central server needed

## For Testing

Since Railway doesn't work for raw P2P:

1. **Local testing**: Works perfectly (mDNS)
2. **Internet testing**: Need ONE person with:
   - Public IP, OR
   - Port forwarding, OR
   - Real VPS (not Railway)

That one person becomes the bootstrap. Everyone else connects through DHT.

## Bottom Line

MRBN is already a true P2P system. We just need to:
1. Deploy ONE node with a real public IP
2. Everyone else connects to it once
3. DHT handles the rest forever

No bridges, no proxies, no complexity needed.
