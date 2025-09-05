{{/*
Expand the name of the chart.
*/}}
{{- define "uveddi.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Create a default fully qualified app name.
We truncate at 63 chars because some Kubernetes name fields are limited to this (by the DNS naming spec).
If release name contains chart name it will be used as a full name.
*/}}
{{- define "uveddi.fullname" -}}
{{- if .Values.fullnameOverride }}
{{- .Values.fullnameOverride | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- $name := default .Chart.Name .Values.nameOverride }}
{{- if contains $name .Release.Name }}
{{- .Release.Name | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- printf "%s-%s" .Release.Name $name | trunc 63 | trimSuffix "-" }}
{{- end }}
{{- end }}
{{- end }}

{{/*
Create chart name and version as used by the chart label.
*/}}
{{- define "uveddi.chart" -}}
{{- printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Common labels
*/}}
{{- define "uveddi.labels" -}}
helm.sh/chart: {{ include "uveddi.chart" . }}
{{ include "uveddi.selectorLabels" . }}
{{- if .Chart.AppVersion }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
{{- end }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
app.kubernetes.io/part-of: uveddi-platform
{{- end }}

{{/*
Selector labels
*/}}
{{- define "uveddi.selectorLabels" -}}
app.kubernetes.io/name: {{ include "uveddi.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end }}

{{/*
Create the name of the service account to use
*/}}
{{- define "uveddi.serviceAccountName" -}}
{{- if .Values.serviceAccount.create }}
{{- default (include "uveddi.fullname" .) .Values.serviceAccount.name }}
{{- else }}
{{- default "default" .Values.serviceAccount.name }}
{{- end }}
{{- end }}

{{/*
Create the image name
*/}}
{{- define "uveddi.image" -}}
{{- $registry := .Values.global.imageRegistry | default .Values.image.registry -}}
{{- $repository := .Values.image.repository -}}
{{- $tag := .Values.image.tag | default .Chart.AppVersion -}}
{{- if $registry }}
{{- printf "%s/%s:%s" $registry $repository $tag }}
{{- else }}
{{- printf "%s:%s" $repository $tag }}
{{- end }}
{{- end }}

{{/*
Create the migration image name
*/}}
{{- define "uveddi.migrationImage" -}}
{{- $registry := .Values.global.imageRegistry | default .Values.migration.image.registry -}}
{{- $repository := .Values.migration.image.repository -}}
{{- $tag := .Values.migration.image.tag | default .Chart.AppVersion -}}
{{- if $registry }}
{{- printf "%s/%s:%s" $registry $repository $tag }}
{{- else }}
{{- printf "%s:%s" $repository $tag }}
{{- end }}
{{- end }}

{{/*
Create the test image name
*/}}
{{- define "uveddi.testImage" -}}
{{- $registry := .Values.global.imageRegistry | default .Values.tests.image.registry -}}
{{- $repository := .Values.tests.image.repository -}}
{{- $tag := .Values.tests.image.tag | default .Chart.AppVersion -}}
{{- if $registry }}
{{- printf "%s/%s:%s" $registry $repository $tag }}
{{- else }}
{{- printf "%s:%s" $repository $tag }}
{{- end }}
{{- end }}

{{/*
Generate environment-specific labels
*/}}
{{- define "uveddi.environmentLabels" -}}
{{- if .Values.environment }}
environment: {{ .Values.environment }}
{{- end }}
{{- end }}

{{/*
Generate pod security context
*/}}
{{- define "uveddi.podSecurityContext" -}}
{{- with .Values.podSecurityContext }}
{{- toYaml . }}
{{- end }}
{{- end }}

{{/*
Generate container security context
*/}}
{{- define "uveddi.securityContext" -}}
{{- with .Values.securityContext }}
{{- toYaml . }}
{{- end }}
{{- end }}

{{/*
Generate resource requirements
*/}}
{{- define "uveddi.resources" -}}
{{- with .Values.resources }}
{{- toYaml . }}
{{- end }}
{{- end }}

{{/*
Generate node selector
*/}}
{{- define "uveddi.nodeSelector" -}}
{{- with .Values.nodeSelector }}
{{- toYaml . }}
{{- end }}
{{- end }}

{{/*
Generate tolerations
*/}}
{{- define "uveddi.tolerations" -}}
{{- with .Values.tolerations }}
{{- toYaml . }}
{{- end }}
{{- end }}

{{/*
Generate affinity
*/}}
{{- define "uveddi.affinity" -}}
{{- with .Values.affinity }}
{{- toYaml . }}
{{- end }}
{{- end }}

{{/*
Generate checksum for config
*/}}
{{- define "uveddi.configChecksum" -}}
{{- $config := include (print .Template.BasePath "/configmap.yaml") . -}}
{{- $config | sha256sum }}
{{- end }}

{{/*
Validate required values
*/}}
{{- define "uveddi.validateValues" -}}
{{- if and .Values.postgresql.enabled .Values.externalSecrets.enabled }}
{{- fail "Cannot enable both postgresql.enabled and externalSecrets.enabled" }}
{{- end }}
{{- if not (or .Values.postgresql.enabled .Values.externalSecrets.enabled) }}
{{- if not .Values.app.config.database }}
{{- fail "Must provide database configuration via postgresql.enabled, externalSecrets.enabled, or app.config.database" }}
{{- end }}
{{- end }}
{{- end }}