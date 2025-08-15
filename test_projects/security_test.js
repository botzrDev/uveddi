// Test file with sensitive information
const API_KEY = 'sk-test-key-123456';
const password = 'admin123';
const SECRET_TOKEN = 'eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9';
const DB_PASSWORD = 'super_secret_password';

function authenticate(user, pass) {
    if (pass === 'hardcoded_password') {
        return 'access_granted';
    }
    return false;
}

const config = {
    database: {
        host: 'localhost',
        user: 'root',
        password: 'root123',  // Another password
        port: 3306
    },
    aws: {
        accessKeyId: 'AKIAIOSFODNN7EXAMPLE',
        secretAccessKey: 'wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY'
    }
};

// Credit card test data
const testCard = '4532-1234-5678-9012';