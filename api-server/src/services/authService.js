class AuthService {
  async registerUser(email, username, password) {
    // For alpha: simulate successful registration
    const mockUser = {
      id: Date.now(),
      email: email,
      username: username,
      full_name: username,
      is_active: true,
      created_at: new Date().toISOString(),
      role: 'alpha_tester'
    };

    const mockToken = `alpha_token_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;

    return {
      user: mockUser,
      access_token: mockToken,
      token_type: 'bearer',
      message: 'Alpha account created successfully'
    };
  }

  async loginUser(email, password) {
    // For alpha: simple validation
    if (password.length < 6) {
      throw new Error('Invalid credentials');
    }

    // Simulate successful login for any valid credentials
    const mockUser = {
      id: Date.now(),
      email: email,
      username: email.split('@')[0],
      full_name: email.split('@')[0],
      is_active: true,
      created_at: new Date().toISOString(),
      role: 'alpha_tester'
    };

    const mockToken = `alpha_token_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;

    return {
      user: mockUser,
      access_token: mockToken,
      token_type: 'bearer',
      message: 'Alpha login successful'
    };
  }
}

module.exports = AuthService;