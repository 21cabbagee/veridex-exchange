/**
 * ⟐ UNIVERSAL SUPABASE CLIENT ADAPTER FOR ALL 42 VENTURES
 * Compatible with vanilla JS (ES Modules / <script>), React, and Node.js.
 * Automatically switches between live Supabase Cloud/Local and Offline Simulator.
 */

const getEnv = (key, fallback = '') => {
  if (typeof process !== 'undefined' && process.env && process.env[key]) return process.env[key];
  if (typeof window !== 'undefined' && window.__ENV__ && window.__ENV__[key]) return window.__ENV__[key];
  if (typeof localStorage !== 'undefined' && localStorage.getItem(key)) return localStorage.getItem(key);
  return fallback;
};

const SUPABASE_URL = getEnv('VITE_SUPABASE_URL', getEnv('NEXT_PUBLIC_SUPABASE_URL', getEnv('SUPABASE_URL', 'https://ijlccbwdocrhbvvqushf.supabase.co')));
const SUPABASE_ANON_KEY = getEnv('VITE_SUPABASE_ANON_KEY', getEnv('NEXT_PUBLIC_SUPABASE_ANON_KEY', getEnv('SUPABASE_ANON_KEY', 'sb_publishable_crK7qnWSFmcMZ3_kd9Wg7Q_ouoswYca')));

class UniversalVenturesClient {
  constructor() {
    this.isLive = Boolean(SUPABASE_URL && SUPABASE_ANON_KEY);
    this.client = null;
    this.init();
  }

  async init() {
    if (this.isLive && typeof window !== 'undefined') {
      try {
        // Dynamically load supabase-js from CDN if not already in window
        if (!window.supabase && !window.createClient) {
          const script = document.createElement('script');
          script.src = 'https://cdn.jsdelivr.net/npm/@supabase/supabase-js@2';
          document.head.appendChild(script);
          await new Promise((resolve) => { script.onload = resolve; });
        }
        const createClient = window.supabase?.createClient || window.createClient;
        if (createClient) {
          this.client = createClient(SUPABASE_URL, SUPABASE_ANON_KEY);
          console.info('[Supabase] Connected to live backend:', SUPABASE_URL);
        }
      } catch (err) {
        console.warn('[Supabase] Failed to initialize live client, using offline fallback:', err);
      }
    }
  }

  // ── Ventures Registry ──
  async getVentures(category = null) {
    try {
      if (this.client) {
        let q = this.client.from('ventures').select('*').order('name');
        if (category) q = q.eq('category', category);
        const { data, error } = await q;
        if (!error && data && data.length > 0) return data;
      }
      if (SUPABASE_URL && SUPABASE_ANON_KEY) {
        let url = `${SUPABASE_URL}/rest/v1/ventures?select=*&order=name.asc`;
        if (category) url += `&category=eq.${encodeURIComponent(category)}`;
        const res = await fetch(url, {
          headers: {
            'apikey': SUPABASE_ANON_KEY,
            'Authorization': `Bearer ${SUPABASE_ANON_KEY}`
          }
        });
        if (res.ok) return await res.json();
      }
    } catch (err) {
      console.warn('[Supabase] Failed to fetch ventures:', err);
    }
    // Static fallback
    return [
      { slug: 'veridex-exchange', name: 'VERIDEX', category: 'prediction', status: 'live' },
      { slug: 'swarm-ops', name: 'SwarmOps', category: 'flagship', status: 'live' },
      { slug: 'operator-academy', name: 'Operator Academy', category: 'flagship', status: 'live' },
      { slug: 'propsoch-studio', name: 'Propsoch Studio', category: 'flagship', status: 'live' },
      { slug: 'founder-os', name: 'Founder OS', category: 'flagship', status: 'live' },
      { slug: 'subarb-engine', name: 'SubArb Engine', category: 'flagship', status: 'live' },
    ];
  }

  // ── Cross-Venture Event Telemetry ──
  async trackEvent(ventureSlug, eventType, payload = {}, userId = null) {
    try {
      if (this.client) {
        const { error } = await this.client.from('ventures_events').insert({
          venture_slug: ventureSlug,
          event_type: eventType,
          payload,
          user_id: userId,
          visibility: 'public'
        });
        if (!error) return true;
      }
      if (SUPABASE_URL && SUPABASE_ANON_KEY) {
        await fetch(`${SUPABASE_URL}/rest/v1/ventures_events`, {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
            'apikey': SUPABASE_ANON_KEY,
            'Authorization': `Bearer ${SUPABASE_ANON_KEY}`,
            'Prefer': 'return=minimal'
          },
          body: JSON.stringify({
            venture_slug: ventureSlug,
            event_type: eventType,
            payload,
            user_id: userId,
            visibility: 'public'
          })
        });
        return true;
      }
    } catch (_) {}
    console.debug(`[Telemetry: ${ventureSlug}] ${eventType}:`, payload);
    return true;
  }

  // ── VERIDEX Domain Helper ──
  async getVeridexMarkets() {
    try {
      if (this.client) {
        const { data, error } = await this.client.from('veridex_markets').select('*').eq('status', 'ACTIVE');
        if (!error && data && data.length > 0) return data;
      }
      if (SUPABASE_URL && SUPABASE_ANON_KEY) {
        const res = await fetch(`${SUPABASE_URL}/rest/v1/veridex_markets?select=*&status=eq.ACTIVE`, {
          headers: {
            'apikey': SUPABASE_ANON_KEY,
            'Authorization': `Bearer ${SUPABASE_ANON_KEY}`
          }
        });
        if (res.ok) return await res.json();
      }
    } catch (err) {
      console.warn('[Supabase] Failed to fetch live markets:', err);
    }
    return null;
  }

  async placeVeridexOrder(order) {
    if (this.client) {
      const { data, error } = await this.client.from('veridex_orders').insert(order).select().single();
      if (!error) return data;
    }
    return { id: 'mock-' + Date.now(), ...order, status: 'OPEN' };
  }

  subscribeVeridexTrades(marketId, callback) {
    if (this.client) {
      return this.client
        .channel(`trades:${marketId}`)
        .on('postgres_changes', { event: 'INSERT', schema: 'public', table: 'veridex_trades', filter: `market_id=eq.${marketId}` }, (payload) => {
          callback(payload.new);
        })
        .subscribe();
    }
  }

  // ── SwarmOps Domain Helper ──
  async getSwarmAgents() {
    if (this.client) {
      const { data, error } = await this.client.from('swarm_agents').select('*').order('agent_index').limit(100);
      if (!error && data) return data;
    }
    return [];
  }
}

// Global Singleton
const universalClient = new UniversalVenturesClient();
if (typeof window !== 'undefined') {
  window.venturesSupabase = universalClient;
}

if (typeof module !== 'undefined' && module.exports) {
  module.exports = { universalClient, UniversalVenturesClient };
}
