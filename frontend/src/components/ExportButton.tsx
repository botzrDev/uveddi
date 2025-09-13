import React, { useState } from 'react';
import {
  Alert,
  Box,
  Button,
  Dialog,
  DialogActions,
  DialogContent,
  DialogContentText,
  DialogTitle,
  FormControl,
  FormControlLabel,
  LinearProgress,
  Radio,
  RadioGroup,
  Snackbar,
  Typography
} from '@mui/material';
import {
  CheckCircleOutline,
  DownloadOutlined,
  ErrorOutline,
  HourglassEmpty
} from '@mui/icons-material';
import { apiService } from '@/services/api';

interface ExportButtonProps {
  reportId?: string;
  variant?: 'button' | 'dialog';
  disabled?: boolean;
}

type ExportFormat = 'markdown' | 'pdf' | 'html';
type ExportStatus = 'idle' | 'exporting' | 'success' | 'error';

export const ExportButton: React.FC<ExportButtonProps> = ({
  reportId,
  variant = 'button',
  disabled = false
}) => {
  const [exportStatus, setExportStatus] = useState<ExportStatus>('idle');
  const [exportFormat, setExportFormat] = useState<ExportFormat>('markdown');
  const [dialogOpen, setDialogOpen] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [successMessage, setSuccessMessage] = useState<string | null>(null);

  const handleExport = async (format: ExportFormat) => {
    console.log('🚀 Export button clicked for format:', format);
    setExportStatus('exporting');
    setError(null);

    try {
      let blob: Blob;
      let filename: string;

      if (reportId) {
        console.log('📊 Exporting specific report:', reportId);
        blob = await apiService.exportReport(reportId, format);
        filename = `uveddi-report-${reportId}.${format}`;
      } else {
        console.log('📊 Exporting latest report...');
        blob = await apiService.exportLatestReport(format);
        filename = `uveddi-analysis-latest.${format}`;
      }

      console.log('✅ Export successful! Blob size:', blob.size, 'bytes');

      // Create download link
      const url = window.URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.style.display = 'none';
      a.href = url;
      a.download = filename;

      console.log('📁 Creating download with filename:', filename);

      document.body.appendChild(a);
      a.click();
      window.URL.revokeObjectURL(url);
      document.body.removeChild(a);

      setExportStatus('success');
      setSuccessMessage(`Report exported successfully as ${format.toUpperCase()}`);
      if (variant === 'dialog') {
        setDialogOpen(false);
      }

      console.log('✅ Download initiated successfully!');
    } catch (error) {
      console.error('❌ Export failed:', error);
      const errorMessage = error instanceof Error ? error.message : 'Export failed';
      setError(errorMessage);
      setExportStatus('error');
    }
  };

  const handleQuickExport = () => {
    if (variant === 'dialog') {
      setDialogOpen(true);
    } else {
      handleExport('markdown');
    }
  };

  const getButtonProps = () => {
    switch (exportStatus) {
      case 'exporting':
        return {
          startIcon: <HourglassEmpty />,
          children: 'Exporting...',
          disabled: true
        };
      case 'success':
        return {
          startIcon: <CheckCircleOutline />,
          children: 'Exported!',
          disabled: false
        };
      case 'error':
        return {
          startIcon: <ErrorOutline />,
          children: 'Export Failed',
          disabled: false
        };
      default:
        return {
          startIcon: <DownloadOutlined />,
          children: 'Export Report',
          disabled
        };
    }
  };

  const buttonProps = getButtonProps();

  return (
    <>
      <Button
        variant="contained"
        onClick={handleQuickExport}
        sx={{
          background: (theme) => {
            if (exportStatus === 'success') {
              return theme.palette.mode === 'light'
                ? 'linear-gradient(135deg, #4caf50 0%, #66bb6a 50%, #81c784 100%)'
                : 'linear-gradient(135deg, #81c784 0%, #66bb6a 50%, #4caf50 100%)';
            } else if (exportStatus === 'error') {
              return theme.palette.mode === 'light'
                ? 'linear-gradient(135deg, #f44336 0%, #ef5350 50%, #e57373 100%)'
                : 'linear-gradient(135deg, #e57373 0%, #ef5350 50%, #f44336 100%)';
            } else {
              return theme.palette.mode === 'light'
                ? 'linear-gradient(135deg, #1565c0 0%, #1976d2 50%, #42a5f5 100%)'
                : 'linear-gradient(135deg, #42a5f5 0%, #1976d2 50%, #1565c0 100%)';
            }
          },
          color: 'white',
          fontWeight: 600,
          fontSize: '0.95rem',
          px: 3,
          py: 1.5,
          borderRadius: 2,
          textTransform: 'none',
          boxShadow: (theme) => {
            const baseColor = exportStatus === 'success' ? '76, 175, 80' :
                             exportStatus === 'error' ? '244, 67, 54' :
                             theme.palette.mode === 'light' ? '25, 118, 210' : '100, 181, 246';
            return `0 4px 15px rgba(${baseColor}, 0.3)`;
          },
          border: 'none',
          position: 'relative',
          overflow: 'hidden',
          opacity: buttonProps.disabled ? 0.8 : 1,
          transform: exportStatus === 'success' ? 'scale(1.05)' : 'scale(1)',
          '&:hover:not(:disabled)': {
            transform: exportStatus === 'success' ? 'translateY(-2px) scale(1.05)' : 'translateY(-2px)',
            boxShadow: (theme) => {
              const baseColor = exportStatus === 'success' ? '76, 175, 80' :
                               exportStatus === 'error' ? '244, 67, 54' :
                               theme.palette.mode === 'light' ? '25, 118, 210' : '100, 181, 246';
              return `0 6px 20px rgba(${baseColor}, 0.4)`;
            },
          },
          transition: 'all 0.3s cubic-bezier(0.4, 0, 0.2, 1)',
        }}
        {...buttonProps}
      />

      {/* Export Format Dialog */}
      <Dialog open={dialogOpen} onClose={() => setDialogOpen(false)} maxWidth="sm" fullWidth>
        <DialogTitle>Export Analysis Report</DialogTitle>
        <DialogContent>
          <DialogContentText sx={{ mb: 3 }}>
            Choose the format for your analysis report export:
          </DialogContentText>

          <FormControl component="fieldset">
            <RadioGroup
              value={exportFormat}
              onChange={(e) => setExportFormat(e.target.value as ExportFormat)}
            >
              <FormControlLabel
                value="markdown"
                control={<Radio />}
                label={
                  <Box>
                    <Typography variant="body1" fontWeight={600}>
                      Markdown (.md)
                    </Typography>
                    <Typography variant="body2" color="text.secondary">
                      Human-readable format with full analysis details
                    </Typography>
                  </Box>
                }
              />
              <FormControlLabel
                value="html"
                control={<Radio />}
                label={
                  <Box>
                    <Typography variant="body1" fontWeight={600}>
                      HTML (.html)
                    </Typography>
                    <Typography variant="body2" color="text.secondary">
                      Web format with interactive elements and styling
                    </Typography>
                  </Box>
                }
              />
              <FormControlLabel
                value="pdf"
                control={<Radio />}
                label={
                  <Box>
                    <Typography variant="body1" fontWeight={600}>
                      PDF (.pdf)
                    </Typography>
                    <Typography variant="body2" color="text.secondary">
                      Print-ready format for sharing and archival
                    </Typography>
                  </Box>
                }
              />
            </RadioGroup>
          </FormControl>

          {exportStatus === 'exporting' && (
            <Box sx={{ mt: 3 }}>
              <Typography variant="body2" color="text.secondary" gutterBottom>
                Generating {exportFormat.toUpperCase()} export...
              </Typography>
              <LinearProgress />
            </Box>
          )}
        </DialogContent>
        <DialogActions>
          <Button
            onClick={() => setDialogOpen(false)}
            disabled={exportStatus === 'exporting'}
          >
            Cancel
          </Button>
          <Button
            onClick={() => handleExport(exportFormat)}
            variant="contained"
            disabled={exportStatus === 'exporting'}
            startIcon={exportStatus === 'exporting' ? <HourglassEmpty /> : <DownloadOutlined />}
          >
            {exportStatus === 'exporting' ? 'Exporting...' : `Export ${exportFormat.toUpperCase()}`}
          </Button>
        </DialogActions>
      </Dialog>

      {/* Success/Error Snackbars */}
      <Snackbar
        open={!!successMessage}
        autoHideDuration={4000}
        onClose={() => setSuccessMessage(null)}
        anchorOrigin={{ vertical: 'bottom', horizontal: 'center' }}
      >
        <Alert
          onClose={() => setSuccessMessage(null)}
          severity="success"
          variant="filled"
        >
          {successMessage}
        </Alert>
      </Snackbar>

      <Snackbar
        open={!!error}
        autoHideDuration={6000}
        onClose={() => setError(null)}
        anchorOrigin={{ vertical: 'bottom', horizontal: 'center' }}
      >
        <Alert
          onClose={() => setError(null)}
          severity="error"
          variant="filled"
        >
          Export failed: {error}
        </Alert>
      </Snackbar>
    </>
  );
};

export default ExportButton;