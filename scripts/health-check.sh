#!/bin/bash
set -euo pipefail

# Health Check Script for Uveddi Production
# Comprehensive health checks for all services

# Configuration
NAMESPACE="${NAMESPACE:-uveddi-prod}"
TIMEOUT="${TIMEOUT:-30}"

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Health check results
HEALTH_STATUS=0

# Check function
check_health() {
    local service=$1
    local endpoint=$2
    local expected_status=${3:-200}

    echo -n "Checking $service... "

    local POD=$(kubectl get pod -l app=$service -n "$NAMESPACE" -o jsonpath='{.items[0].metadata.name}' 2>/dev/null)

    if [[ -z "$POD" ]]; then
        echo -e "${RED}✗ No pod found${NC}"
        HEALTH_STATUS=1
        return
    fi

    local response=$(kubectl exec -n "$NAMESPACE" "$POD" -- curl -s -o /dev/null -w "%{http_code}" -m "$TIMEOUT" "$endpoint" 2>/dev/null || echo "000")

    if [[ "$response" == "$expected_status" ]]; then
        echo -e "${GREEN}✓ Healthy (HTTP $response)${NC}"
    else
        echo -e "${RED}✗ Unhealthy (HTTP $response)${NC}"
        HEALTH_STATUS=1
    fi
}

# Check pod status
check_pods() {
    echo "=== Pod Status ==="
    kubectl get pods -n "$NAMESPACE" --no-headers | while read line; do
        local name=$(echo "$line" | awk '{print $1}')
        local ready=$(echo "$line" | awk '{print $2}')
        local status=$(echo "$line" | awk '{print $3}')

        if [[ "$status" == "Running" ]]; then
            echo -e "${GREEN}✓${NC} $name: $ready ($status)"
        else
            echo -e "${RED}✗${NC} $name: $ready ($status)"
            HEALTH_STATUS=1
        fi
    done
    echo
}

# Check services
check_services() {
    echo "=== Service Health Checks ==="

    # API health
    check_health "uveddi-api" "http://localhost:8080/health"
    check_health "uveddi-api" "http://localhost:8080/health/database"
    check_health "uveddi-api" "http://localhost:9090/metrics"

    # Frontend health
    check_health "uveddi-frontend" "http://localhost:8001/health"

    # Rendering service health
    check_health "rendering-service" "http://localhost:3001/health"

    echo
}

# Check database
check_database() {
    echo "=== Database Status ==="

    local POD=$(kubectl get pod -l app=postgres -n "$NAMESPACE" -o jsonpath='{.items[0].metadata.name}' 2>/dev/null)

    if [[ -n "$POD" ]]; then
        if kubectl exec -n "$NAMESPACE" "$POD" -- pg_isready -U uveddi -d uveddi_prod &>/dev/null; then
            echo -e "${GREEN}✓ PostgreSQL is ready${NC}"
        else
            echo -e "${RED}✗ PostgreSQL is not ready${NC}"
            HEALTH_STATUS=1
        fi
    else
        echo -e "${YELLOW}⚠ PostgreSQL pod not found (may be using external database)${NC}"
    fi

    echo
}

# Check Redis
check_redis() {
    echo "=== Cache Status ==="

    local POD=$(kubectl get pod -l app=redis -n "$NAMESPACE" -o jsonpath='{.items[0].metadata.name}' 2>/dev/null)

    if [[ -n "$POD" ]]; then
        if kubectl exec -n "$NAMESPACE" "$POD" -- redis-cli ping &>/dev/null; then
            echo -e "${GREEN}✓ Redis is ready${NC}"
        else
            echo -e "${RED}✗ Redis is not ready${NC}"
            HEALTH_STATUS=1
        fi
    else
        echo -e "${YELLOW}⚠ Redis pod not found (may be using external cache)${NC}"
    fi

    echo
}

# Check ingress
check_ingress() {
    echo "=== Ingress Status ==="

    kubectl get ingress -n "$NAMESPACE" --no-headers | while read line; do
        local name=$(echo "$line" | awk '{print $1}')
        local hosts=$(echo "$line" | awk '{print $3}')
        local address=$(echo "$line" | awk '{print $4}')

        if [[ -n "$address" ]]; then
            echo -e "${GREEN}✓${NC} $name: $hosts -> $address"
        else
            echo -e "${YELLOW}⚠${NC} $name: $hosts (no address assigned)"
        fi
    done

    echo
}

# Check certificates
check_certificates() {
    echo "=== SSL Certificate Status ==="

    kubectl get certificate -n "$NAMESPACE" --no-headers 2>/dev/null | while read line; do
        local name=$(echo "$line" | awk '{print $1}')
        local ready=$(echo "$line" | awk '{print $2}')

        if [[ "$ready" == "True" ]]; then
            echo -e "${GREEN}✓${NC} $name: Ready"
        else
            echo -e "${RED}✗${NC} $name: Not Ready"
            HEALTH_STATUS=1
        fi
    done || echo -e "${YELLOW}⚠ No certificates found (cert-manager may not be installed)${NC}"

    echo
}

# Check resource usage
check_resources() {
    echo "=== Resource Usage ==="

    kubectl top pods -n "$NAMESPACE" --no-headers 2>/dev/null | while read line; do
        local name=$(echo "$line" | awk '{print $1}')
        local cpu=$(echo "$line" | awk '{print $2}')
        local memory=$(echo "$line" | awk '{print $3}')

        echo "$name: CPU=$cpu, Memory=$memory"
    done || echo -e "${YELLOW}⚠ Metrics server not available${NC}"

    echo
}

# Check recent events
check_events() {
    echo "=== Recent Events ==="

    kubectl get events -n "$NAMESPACE" --sort-by='.lastTimestamp' | tail -5

    echo
}

# Main health check
main() {
    echo "==========================================="
    echo "Uveddi Production Health Check"
    echo "Namespace: $NAMESPACE"
    echo "Time: $(date)"
    echo "==========================================="
    echo

    check_pods
    check_services
    check_database
    check_redis
    check_ingress
    check_certificates
    check_resources
    check_events

    echo "==========================================="
    if [[ $HEALTH_STATUS -eq 0 ]]; then
        echo -e "${GREEN}Overall Status: HEALTHY${NC}"
    else
        echo -e "${RED}Overall Status: UNHEALTHY${NC}"
    fi
    echo "==========================================="

    exit $HEALTH_STATUS
}

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --namespace)
            NAMESPACE="$2"
            shift 2
            ;;
        --timeout)
            TIMEOUT="$2"
            shift 2
            ;;
        --help)
            echo "Usage: $0 [OPTIONS]"
            echo "Options:"
            echo "  --namespace NAME  Kubernetes namespace (default: uveddi-prod)"
            echo "  --timeout SECONDS  Health check timeout (default: 30)"
            echo "  --help            Show this help message"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

main