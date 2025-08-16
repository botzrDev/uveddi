import React from 'react';
import {
  Box,
  Typography,
  Alert,
} from '@mui/material';

function ReportsListPage() {
  return (
    <Box>
      <Typography variant="h3" component="h1" gutterBottom>
        All Reports
      </Typography>
      
      <Alert severity="info" sx={{ mt: 2 }}>
        <Typography variant="h6" gutterBottom>
          Coming Soon
        </Typography>
        <Typography variant="body2">
          This page will show a list of all available analysis reports.
        </Typography>
        <Typography variant="body2" sx={{ mt: 1 }}>
          Features will include:
        </Typography>
        <ul>
          <li>Searchable and sortable report list</li>
          <li>Filter by date, project, or status</li>
          <li>Quick preview of report summaries</li>
          <li>Export and sharing options</li>
        </ul>
      </Alert>
    </Box>
  );
}

export default ReportsListPage;