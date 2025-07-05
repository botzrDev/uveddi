#!/bin/bash
# install_cypress.sh
# Installs Cypress as a dev dependency in the frontend directory
set -e

cd "$(dirname "$0")/../frontend"

if ! command -v npm &> /dev/null; then
  echo "npm is not installed. Please install Node.js and npm first."
  exit 1
fi

npm install cypress --save-dev

echo "Cypress installation complete. Run 'npx cypress open' in the frontend directory to launch."
