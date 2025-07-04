import React from 'react';

const Footer: React.FC = () => (
  <footer className="bg-secondary-900 border-t border-secondary-800 py-12">
    <div className="container mx-auto px-6">
      <div className="grid md:grid-cols-4 gap-8">
        <div>
          <div className="flex items-center mb-4" style={{ gap: '2px' }}>
            <img src="/logo(blue).png" alt="Uveddi Logo" className="w-10 h-10 rounded-lg" />
            <span className="font-mono text-2xl font-bold bg-gradient-to-r from-white to-primary-200 bg-clip-text text-transparent tracking-wider">
              veddi
            </span>
          </div>
          <p className="text-secondary-400 text-sm">
            High-level code analysis for the modern developer.
          </p>
        </div>
        <div>
          <h4 className="font-semibold text-white mb-4">Product</h4>
          <ul className="space-y-2 text-secondary-400 text-sm">
            <li><a href="#" className="hover:text-primary-400 transition-colors">Features</a></li>
            <li><a href="#" className="hover:text-primary-400 transition-colors">Pricing</a></li>
            <li><a href="#" className="hover:text-primary-400 transition-colors">Integrations</a></li>
            <li><a href="#" className="hover:text-primary-400 transition-colors">API</a></li>
          </ul>
        </div>
        <div>
          <h4 className="font-semibold text-white mb-4">Resources</h4>
          <ul className="space-y-2 text-secondary-400 text-sm">
            <li><a href="#" className="hover:text-primary-400 transition-colors">Documentation</a></li>
            <li><a href="#" className="hover:text-primary-400 transition-colors">Blog</a></li>
            <li><a href="#" className="hover:text-primary-400 transition-colors">Community</a></li>
            <li><a href="#" className="hover:text-primary-400 transition-colors">Support</a></li>
          </ul>
        </div>
        <div>
          <h4 className="font-semibold text-white mb-4">Company</h4>
          <ul className="space-y-2 text-secondary-400 text-sm">
            <li><a href="#" className="hover:text-primary-400 transition-colors">About</a></li>
            <li><a href="#" className="hover:text-primary-400 transition-colors">Privacy Policy</a></li>
            <li><a href="#" className="hover:text-primary-400 transition-colors">Terms of Service</a></li>
            <li><a href="#" className="hover:text-primary-400 transition-colors">Contact</a></li>
          </ul>
        </div>
      </div>
      <div className="border-t border-secondary-800 mt-8 pt-8 text-center text-secondary-400 text-sm">
        <p>&copy; 2025 uveddi. All rights reserved.</p>
      </div>
    </div>
  </footer>
);

export default Footer;
