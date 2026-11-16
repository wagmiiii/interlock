import React from 'react';

export default function App(): React.JSX.Element {
  return (
    <div style={{ fontFamily: 'sans-serif', maxWidth: 800, margin: '40px auto', padding: '0 20px' }}>
      <h1>Interlock</h1>
      <p style={{ color: '#555' }}>
        Non-custodial cost engine and atomic oracle guard for Solana perpetuals.
      </p>

      <div style={{ padding: 16, backgroundColor: '#f4f4f4', borderRadius: 8, marginTop: 24 }}>
        <h3>Critical Boundary</h3>
        <p>
          Interlock protects user-signed transactions only. It does not stop keeper liquidations.
        </p>
      </div>

      <div style={{ marginTop: 24 }}>
        <h3>Cost Calculator</h3>
        <p>Market: SOL-PERP | Read-only mode</p>
      </div>
    </div>
  );
}
