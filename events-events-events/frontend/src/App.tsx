import React, { useState, useEffect } from 'react';
import { connect, JSONCodec } from 'nats.ws';
import { ShoppingCart, Activity, Package, CreditCard, CheckCircle } from 'lucide-react';

const jc = JSONCodec();

interface Event {
  id: string;
  type: string;
  data: any;
  timestamp: Date;
}

const App: React.FC = () => {
  const [events, setEvents] = useState<Event[]>([]);
  const [customerID, setCustomerID] = useState('customer-1');
  const [amount, setAmount] = useState('49.99');
  const [status, setStatus] = useState<'idle' | 'loading' | 'success' | 'error'>('idle');

  useEffect(() => {
    let nc: any;
    const initNats = async () => {
      try {
        nc = await connect({ servers: ['ws://localhost:9222'] });
        console.log('Connected to NATS');
        
        // Subscribe to all EDA events
        const sub = nc.subscribe('>');
        (async () => {
          for await (const m of sub) {
            const data = jc.decode(m.data);
            const newEvent: Event = {
              id: Math.random().toString(36).substr(2, 9),
              type: m.subject,
              data: data,
              timestamp: new Date()
            };
            setEvents(prev => [newEvent, ...prev].slice(0, 50));
          }
        })();
      } catch (err) {
        console.error('NATS Connection Error:', err);
      }
    };

    initNats();
    return () => { if (nc) nc.close(); };
  }, []);

  const placeOrder = async () => {
    setStatus('loading');
    try {
      // ConnectRPC supports simple JSON POST to /service.name/MethodName
      const response = await fetch('http://localhost:8080/order.v1.OrderService/CreateOrder', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Connect-Protocol-Version': '1'
        },
        body: JSON.stringify({
          customer_id: customerID,
          amount: parseFloat(amount)
        })
      });

      if (response.ok) {
        setStatus('success');
        setTimeout(() => setStatus('idle'), 2000);
      } else {
        setStatus('error');
      }
    } catch (err) {
      console.error('Order Error:', err);
      setStatus('error');
    }
  };

  const getEventIcon = (type: string) => {
    if (type.includes('created')) return <Package className="text-blue-400" />;
    if (type.includes('processed')) return <CreditCard className="text-purple-400" />;
    if (type.includes('status')) return <CheckCircle className="text-green-400" />;
    return <Activity className="text-slate-400" />;
  };

  return (
    <div className="max-w-6xl mx-auto p-8">
      <header className="flex justify-between items-center mb-12">
        <h1 className="text-3xl font-bold flex items-center gap-3">
          <Activity className="text-blue-500" />
          EDA Dashboard
        </h1>
        <div className="flex gap-4">
          <div className="bg-slate-800 p-4 rounded-lg border border-slate-700">
             <p className="text-xs text-slate-400 uppercase tracking-wider font-semibold">NATS Status</p>
             <p className="text-green-400 font-mono flex items-center gap-2">
                <span className="w-2 h-2 bg-green-400 rounded-full animate-pulse"></span>
                Connected (WS:9222)
             </p>
          </div>
        </div>
      </header>

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-8">
        {/* Place Order Panel */}
        <div className="bg-slate-800 p-6 rounded-xl border border-slate-700 shadow-xl h-fit">
          <h2 className="text-xl font-semibold mb-6 flex items-center gap-2">
            <ShoppingCart className="text-blue-400" />
            Place New Order
          </h2>
          <div className="space-y-4">
            <div>
              <label className="block text-sm font-medium text-slate-400 mb-1">Customer ID</label>
              <input 
                type="text" 
                value={customerID}
                onChange={e => setCustomerID(e.target.value)}
                className="w-full bg-slate-900 border border-slate-700 rounded-lg p-2.5 focus:ring-2 focus:ring-blue-500 outline-none"
              />
            </div>
            <div>
              <label className="block text-sm font-medium text-slate-400 mb-1">Amount ($)</label>
              <input 
                type="number" 
                value={amount}
                onChange={e => setAmount(e.target.value)}
                className="w-full bg-slate-900 border border-slate-700 rounded-lg p-2.5 focus:ring-2 focus:ring-blue-500 outline-none"
              />
            </div>
            <button 
              onClick={placeOrder}
              disabled={status === 'loading'}
              className={`w-full py-3 rounded-lg font-bold transition-all ${
                status === 'loading' ? 'bg-slate-700' : 
                status === 'success' ? 'bg-green-600' :
                status === 'error' ? 'bg-red-600' : 'bg-blue-600 hover:bg-blue-500'
              }`}
            >
              {status === 'loading' ? 'Processing...' : status === 'success' ? 'Order Placed!' : 'Place Order'}
            </button>
          </div>
        </div>

        {/* Live Events Stream */}
        <div className="lg:col-span-2 bg-slate-800 p-6 rounded-xl border border-slate-700 shadow-xl min-h-[500px]">
          <h2 className="text-xl font-semibold mb-6 flex items-center gap-2">
            <Activity className="text-purple-400" />
            Real-time Event Stream
          </h2>
          <div className="space-y-3">
            {events.length === 0 && (
              <div className="flex flex-col items-center justify-center py-20 text-slate-500 italic">
                <p>Waiting for events...</p>
                <p className="text-sm">Place an order to see the flow in action.</p>
              </div>
            )}
            {events.map((event) => (
              <div key={event.id} className="bg-slate-900 p-4 rounded-lg border border-slate-700 flex items-start gap-4 animate-in fade-in slide-in-from-top-2 duration-300">
                <div className="mt-1">
                  {getEventIcon(event.type)}
                </div>
                <div className="flex-1">
                  <div className="flex justify-between items-center mb-1">
                    <span className="font-mono text-blue-400 text-sm">{event.type}</span>
                    <span className="text-xs text-slate-500">{event.timestamp.toLocaleTimeString()}</span>
                  </div>
                  <pre className="text-xs bg-slate-950 p-2 rounded border border-slate-800 overflow-x-auto text-slate-300">
                    {JSON.stringify(event.data, null, 2)}
                  </pre>
                </div>
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
};

export default App;
