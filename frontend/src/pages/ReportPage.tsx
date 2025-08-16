import React from 'react';
import { useParams } from 'react-router-dom';
import {
  Box,
  Typography,
  Alert,
} from '@mui/material';

function ReportPage() {
  const { reportId } = useParams<{ reportId: string }>();

  return (
    <Box>
      <Typography variant="h3" component="h1" gutterBottom>
        Report Details
      </Typography>
      
      <Alert severity="info" sx={{ mt: 2 }}>
        <Typography variant="h6" gutterBottom>
          Coming Soon
        </Typography>
        <Typography variant="body2">
          Detailed report view for report ID: {reportId}
        </Typography>
        <Typography variant="body2" sx={{ mt: 1 }}>
          This page will include:
        </Typography>
        <ul>
          <li>Detailed findings list with filtering and search</li>
          <li>Interactive dependency graphs</li>
          <li>Mermaid diagrams</li>
          <li>Code snippets and recommendations</li>
          <li>AI insights and suggestions</li>
        </ul>
      </Alert>
    </Box>
  );
}

export default ReportPage;