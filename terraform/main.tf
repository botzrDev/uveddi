# Main Terraform configuration for Uveddi infrastructure
terraform {
  required_version = ">= 1.5"
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"
    }
    kubernetes = {
      source  = "hashicorp/kubernetes"
      version = "~> 2.23"
    }
    helm = {
      source  = "hashicorp/helm"
      version = "~> 2.11"
    }
    random = {
      source  = "hashicorp/random"
      version = "~> 3.1"
    }
    tls = {
      source  = "hashicorp/tls"
      version = "~> 4.0"
    }
  }
  
  # Backend configuration (uncomment and configure for your setup)
  # backend "s3" {
  #   bucket         = "uveddi-terraform-state"
  #   key            = "infrastructure/terraform.tfstate"
  #   region         = "us-west-2"
  #   encrypt        = true
  #   dynamodb_table = "terraform-locks"
  # }
}

# Configure AWS Provider
provider "aws" {
  region = var.aws_region
  
  default_tags {
    tags = {
      Project     = "uveddi"
      Environment = var.environment
      ManagedBy   = "terraform"
      CreatedBy   = "uveddi-platform"
    }
  }
}

# Data sources
data "aws_availability_zones" "available" {
  state = "available"
}

data "aws_caller_identity" "current" {}

# Local values
locals {
  name_prefix = "uveddi-${var.environment}"
  azs         = slice(data.aws_availability_zones.available.names, 0, 3)
  
  common_tags = {
    Project     = "uveddi"
    Environment = var.environment
    ManagedBy   = "terraform"
    CreatedBy   = "uveddi-platform"
  }
}

# Networking
module "networking" {
  source = "./modules/networking"
  
  name_prefix = local.name_prefix
  environment = var.environment
  
  vpc_cidr = var.vpc_cidr
  azs      = local.azs
  
  enable_nat_gateway   = true
  enable_vpc_endpoints = var.enable_vpc_endpoints
  
  tags = local.common_tags
}

# EKS Cluster
module "eks" {
  source = "./modules/eks"
  
  name_prefix = local.name_prefix
  environment = var.environment
  
  cluster_version = var.eks_cluster_version
  
  vpc_id                    = module.networking.vpc_id
  subnet_ids               = module.networking.private_subnets
  control_plane_subnet_ids = module.networking.public_subnets
  
  # Node groups configuration
  node_groups = var.eks_node_groups
  
  # Add-ons
  enable_cluster_autoscaler     = true
  enable_aws_load_balancer_controller = true
  enable_external_dns          = var.enable_external_dns
  enable_cert_manager          = true
  
  # Security
  enable_irsa = true
  
  tags = local.common_tags
  
  depends_on = [module.networking]
}

# RDS Database (PostgreSQL)
module "rds" {
  source = "./modules/rds"
  count  = var.enable_rds ? 1 : 0
  
  name_prefix = local.name_prefix
  environment = var.environment
  
  vpc_id          = module.networking.vpc_id
  subnet_ids      = module.networking.database_subnets
  security_groups = [module.networking.database_security_group_id]
  
  # Database configuration
  engine_version = var.rds_engine_version
  instance_class = var.rds_instance_class
  
  allocated_storage     = var.rds_allocated_storage
  max_allocated_storage = var.rds_max_allocated_storage
  
  database_name = "uveddi"
  username      = "uveddi"
  
  # Backup and maintenance
  backup_retention_period = var.rds_backup_retention
  backup_window          = "03:00-04:00"
  maintenance_window     = "sun:04:00-sun:05:00"
  
  # High availability
  multi_az = var.environment == "production"
  
  # Performance Insights
  performance_insights_enabled = var.environment == "production"
  
  tags = local.common_tags
}

# Redis Cache (ElastiCache)
module "redis" {
  source = "./modules/redis"
  count  = var.enable_redis ? 1 : 0
  
  name_prefix = local.name_prefix
  environment = var.environment
  
  vpc_id     = module.networking.vpc_id
  subnet_ids = module.networking.cache_subnets
  
  # Cache configuration
  node_type               = var.redis_node_type
  num_cache_clusters      = var.redis_num_clusters
  parameter_group_version = var.redis_parameter_group_version
  
  # High availability
  automatic_failover_enabled = var.environment == "production"
  multi_az_enabled          = var.environment == "production"
  
  # Security
  at_rest_encryption_enabled  = true
  transit_encryption_enabled  = true
  
  tags = local.common_tags
}

# Monitoring and Observability
module "monitoring" {
  source = "./modules/monitoring"
  count  = var.enable_monitoring ? 1 : 0
  
  name_prefix = local.name_prefix
  environment = var.environment
  
  # EKS cluster info
  eks_cluster_name = module.eks.cluster_name
  eks_cluster_endpoint = module.eks.cluster_endpoint
  
  # CloudWatch configuration
  log_retention_days = var.cloudwatch_log_retention
  
  # SNS for alerts
  alert_email = var.alert_email
  
  tags = local.common_tags
}

# S3 Bucket for application data/backups
resource "aws_s3_bucket" "app_storage" {
  bucket = "${local.name_prefix}-app-storage"
  
  tags = merge(local.common_tags, {
    Name = "${local.name_prefix}-app-storage"
  })
}

resource "aws_s3_bucket_versioning" "app_storage" {
  bucket = aws_s3_bucket.app_storage.id
  versioning_configuration {
    status = "Enabled"
  }
}

resource "aws_s3_bucket_server_side_encryption_configuration" "app_storage" {
  bucket = aws_s3_bucket.app_storage.id
  
  rule {
    apply_server_side_encryption_by_default {
      sse_algorithm = "AES256"
    }
  }
}

resource "aws_s3_bucket_public_access_block" "app_storage" {
  bucket = aws_s3_bucket.app_storage.id
  
  block_public_acls       = true
  block_public_policy     = true
  ignore_public_acls      = true
  restrict_public_buckets = true
}