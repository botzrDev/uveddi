import React from 'react';

const TestPage: React.FC = () => {
  return (
    <div className="min-h-screen bg-gray-900 text-white p-8">
      <div className="max-w-4xl mx-auto">
        <h1 className="text-4xl font-bold text-green-400 mb-8">
          Tailwind CSS Test Page
        </h1>
        
        <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
          <div className="bg-gray-800 p-6 rounded-lg border border-gray-700">
            <h2 className="text-2xl font-semibold text-white mb-4">Card Example</h2>
            <p className="text-gray-300 mb-4">
              This is a test card to verify Tailwind CSS is working properly.
            </p>
            <button className="bg-green-600 hover:bg-green-700 text-white font-bold py-2 px-4 rounded">
              Test Button
            </button>
          </div>
          
          <div className="bg-blue-800 p-6 rounded-lg">
            <h2 className="text-2xl font-semibold text-white mb-4">Color Test</h2>
            <div className="space-y-2">
              <div className="bg-red-500 p-2 rounded text-white text-center">Red</div>
              <div className="bg-green-500 p-2 rounded text-white text-center">Green</div>
              <div className="bg-blue-500 p-2 rounded text-white text-center">Blue</div>
              <div className="bg-yellow-500 p-2 rounded text-black text-center">Yellow</div>
            </div>
          </div>
        </div>
        
        <div className="mt-8 p-4 bg-gray-800 rounded-lg border-l-4 border-green-500">
          <h3 className="text-lg font-semibold text-green-400 mb-2">Status</h3>
          <p className="text-gray-300">
            If you can see proper colors, spacing, and typography, Tailwind CSS is working correctly!
          </p>
        </div>
      </div>
    </div>
  );
};

export default TestPage;