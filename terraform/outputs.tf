# Outputs for Uveddi infrastructure

# Networking outputs
output "vpc_id" {
  description = "ID of the VPC"
  value       = module.networking.vpc_id
}

output "vpc_cidr" {
  description = "CIDR block of the VPC"
  value       = module.networking.vpc_cidr
}

output "private_subnets" {
  description = "List of IDs of private subnets"
  value       = module.networking.private_subnets
}

output "public_subnets" {
  description = "List of IDs of public subnets"
  value       = module.networking.public_subnets
}

output "database_subnets" {
  description = "List of IDs of database subnets"
  value       = module.networking.database_subnets
}

# EKS outputs
output "eks_cluster_name" {
  description = "Name of the EKS cluster"
  value       = module.eks.cluster_name
}

output "eks_cluster_endpoint" {
  description = "Endpoint for EKS cluster"
  value       = module.eks.cluster_endpoint
  sensitive   = true
}

output "eks_cluster_version" {
  description = "Version of the EKS cluster"
  value       = module.eks.cluster_version
}

output "eks_cluster_security_group_id" {
  description = "Security group ID attached to the EKS cluster"
  value       = module.eks.cluster_security_group_id
}

output "eks_node_groups" {
  description = "EKS node groups information"
  value       = module.eks.node_groups
}

output "eks_cluster_certificate_authority_data" {
  description = "Base64 encoded certificate data required to communicate with the cluster"
  value       = module.eks.cluster_certificate_authority_data
  sensitive   = true
}

# Database outputs
output "rds_endpoint" {
  description = "RDS instance endpoint"
  value       = var.enable_rds ? module.rds[0].db_instance_endpoint : null
}

output "rds_database_name" {
  description = "RDS database name"
  value       = var.enable_rds ? module.rds[0].db_instance_name : null
}

output "rds_username" {
  description = "RDS database username"
  value       = var.enable_rds ? module.rds[0].db_instance_username : null
  sensitive   = true
}

output "rds_password" {
  description = "RDS database password"
  value       = var.enable_rds ? module.rds[0].db_instance_password : null
  sensitive   = true
}

# Redis outputs
output "redis_endpoint" {
  description = "Redis cluster endpoint"
  value       = var.enable_redis ? module.redis[0].primary_endpoint : null
}

output "redis_port" {
  description = "Redis cluster port"
  value       = var.enable_redis ? module.redis[0].port : null
}

# S3 outputs
output "s3_bucket_name" {
  description = "Name of the S3 bucket for application storage"
  value       = aws_s3_bucket.app_storage.bucket
}

output "s3_bucket_arn" {
  description = "ARN of the S3 bucket for application storage"
  value       = aws_s3_bucket.app_storage.arn
}

# Monitoring outputs
output "cloudwatch_log_group" {
  description = "CloudWatch log group for application logs"
  value       = var.enable_monitoring ? module.monitoring[0].log_group_name : null
}

output "sns_topic_arn" {
  description = "SNS topic ARN for alerts"
  value       = var.enable_monitoring ? module.monitoring[0].sns_topic_arn : null
}

# kubectl configuration command
output "kubectl_config_command" {
  description = "Command to configure kubectl"
  value       = "aws eks update-kubeconfig --region ${var.aws_region} --name ${module.eks.cluster_name}"
}

# Helm installation commands
output "helm_install_commands" {
  description = "Commands to install Uveddi using Helm"
  value = {
    staging = "helm upgrade --install uveddi ./helm/uveddi --namespace uveddi --create-namespace --values ./helm/uveddi/values-staging.yaml"
    production = "helm upgrade --install uveddi ./helm/uveddi --namespace uveddi --create-namespace --values ./helm/uveddi/values-production.yaml"
  }
}

# Environment-specific connection information
output "connection_info" {
  description = "Connection information for the deployed infrastructure"
  value = {
    cluster_name = module.eks.cluster_name
    region      = var.aws_region
    environment = var.environment
    
    database_url = var.enable_rds ? "postgresql://${module.rds[0].db_instance_username}:${module.rds[0].db_instance_password}@${module.rds[0].db_instance_endpoint}/uveddi" : null
    redis_url    = var.enable_redis ? "redis://${module.redis[0].primary_endpoint}:${module.redis[0].port}" : null
    
    s3_bucket = aws_s3_bucket.app_storage.bucket
    
    kubectl_command = "aws eks update-kubeconfig --region ${var.aws_region} --name ${module.eks.cluster_name}"
  }
  sensitive = true
}