import React from 'react';

const SimpleTest: React.FC = () => {
  return (
    <div style={{ padding: '20px', backgroundColor: '#1a202c', color: 'white', minHeight: '100vh' }}>
      <h1 style={{ color: '#10b981', fontSize: '2rem', marginBottom: '1rem' }}>
        Simple Test Page
      </h1>
      <p style={{ marginBottom: '1rem' }}>
        If you can see this, React is working!
      </p>
      <div style={{ backgroundColor: '#2d3748', padding: '1rem', borderRadius: '8px' }}>
        <h2 style={{ color: '#3182ce' }}>Test Features:</h2>
        <ul>
          <li>✅ React rendering</li>
          <li>✅ TypeScript compilation</li>
          <li>✅ Basic styling</li>
        </ul>
      </div>
      <div style={{ marginTop: '2rem' }}>
        <a href="/" style={{ color: '#10b981', textDecoration: 'underline' }}>
          ← Back to Home
        </a>
      </div>
    </div>
  );
};

export default SimpleTest;