// Main script demonstrating usage of problematic code
const { UniversalApplicationController } = require('./src/god_object');

function main() {
    const controller = new UniversalApplicationController();
    
    // Use the god object
    const requestData = {
        id: 'req_123',
        dataKey: 'test_key',
        dataValue: 'test_value',
        apiUrl: 'https://api.example.com/data',
        componentId: 'main_component',
        componentProps: { visible: true }
    };
    
    controller.handleRequest(requestData)
        .then(result => {
            console.log(`Request result: ${JSON.stringify(result)}`);
            
            // This uses active code
            console.log(`Metrics count: ${controller.getMetricValues('requests_handled').length}`);
        })
        .catch(error => {
            console.error(`Error: ${error.message}`);
        });
}

main();