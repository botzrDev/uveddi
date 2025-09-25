const AuthService = require('../services/authService');

class AuthController {
  constructor() {
    this.authService = new AuthService();
  }

  register = async (req, res) => {
    try {
      const { email, username, password } = req.body;

      // Basic validation
      if (!email || !username || !password) {
        return res.status(400).json({
          detail: 'Email, username, and password are required'
        });
      }

      if (password.length < 6) {
        return res.status(400).json({
          detail: 'Password must be at least 6 characters'
        });
      }

      const result = await this.authService.registerUser(email, username, password);
      res.status(201).json(result);

    } catch (error) {
      res.status(500).json({
        detail: 'Registration failed',
        error: error.message
      });
    }
  };

  login = async (req, res) => {
    try {
      const { email, password } = req.body;

      // Basic validation
      if (!email || !password) {
        return res.status(400).json({
          detail: 'Email and password are required'
        });
      }

      const result = await this.authService.loginUser(email, password);
      res.json(result);

    } catch (error) {
      if (error.message === 'Invalid credentials') {
        return res.status(401).json({
          detail: 'Invalid email or password'
        });
      }

      res.status(500).json({
        detail: 'Login failed',
        error: error.message
      });
    }
  };

  logout = (req, res) => {
    try {
      // For alpha: simple logout response
      res.json({
        message: 'Logged out successfully',
        timestamp: new Date().toISOString()
      });
    } catch (error) {
      res.status(500).json({
        detail: 'Logout failed',
        error: error.message
      });
    }
  };

  getCurrentUser = (req, res) => {
    try {
      const authHeader = req.headers.authorization;

      if (!authHeader || !authHeader.startsWith('Bearer ')) {
        return res.status(401).json({
          detail: 'Authentication required'
        });
      }

      // For alpha: return mock user data
      res.json({
        id: 1,
        username: 'alpha_tester',
        email: 'alpha@uveddi.dev',
        full_name: 'Alpha Tester',
        is_active: true,
        role: 'alpha_tester',
        created_at: new Date().toISOString(),
        preferences: {
          theme: 'system',
          notifications: true
        }
      });
    } catch (error) {
      res.status(500).json({
        detail: 'Failed to get user data',
        error: error.message
      });
    }
  };
}

module.exports = AuthController;