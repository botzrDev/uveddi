import React from 'react';
import { screen, fireEvent, waitFor } from '@testing-library/react';
import { render, createMockFile } from '../../../test-utils/test-utils';
import { ExportHub } from '../ExportHub';
import { mockInteractiveReport } from '../../../test-utils/mocks/mockData';

// Mock HTML2Canvas and jsPDF
jest.mock('html2canvas', () => jest.fn(() => 
  Promise.resolve({
    toDataURL: () => 'data:image/png;base64,mock-image-data'
  })
));

jest.mock('jspdf', () => ({
  jsPDF: jest.fn().mockImplementation(() => ({
    addImage: jest.fn(),
    save: jest.fn(),
    text: jest.fn(),
    setFontSize: jest.fn(),
    setFont: jest.fn(),
    addPage: jest.fn(),
    internal: {
      pageSize: { getWidth: () => 210, getHeight: () => 297 }
    }
  }))
}));

describe('ExportHub', () => {
  const mockProps = {
    reportData: mockInteractiveReport,
    onExportComplete: jest.fn(),
    onExportError: jest.fn(),
    availableFormats: ['pdf', 'png', 'svg', 'json', 'csv', 'xlsx'],
  };

  beforeEach(() => {
    jest.clearAllMocks();
    
    // Mock URL.createObjectURL
    global.URL.createObjectURL = jest.fn(() => 'blob:mock-url');
    global.URL.revokeObjectURL = jest.fn();
    
    // Mock document.createElement for download link
    const mockLink = {
      click: jest.fn(),
      download: '',
      href: '',
      style: { display: '' }
    };
    jest.spyOn(document, 'createElement').mockImplementation((tag) => {
      if (tag === 'a') return mockLink as any;
      return document.createElement(tag);
    });
    
    document.body.appendChild = jest.fn();
    document.body.removeChild = jest.fn();
  });

  it('renders export hub component', () => {
    render(<ExportHub {...mockProps} />);
    
    expect(screen.getByText('Export Hub')).toBeInTheDocument();
    expect(screen.getByText('Download Reports & Visualizations')).toBeInTheDocument();
  });

  it('displays available export formats', () => {
    render(<ExportHub {...mockProps} />);
    
    expect(screen.getByText('PDF Report')).toBeInTheDocument();
    expect(screen.getByText('PNG Image')).toBeInTheDocument();
    expect(screen.getByText('SVG Vector')).toBeInTheDocument();
    expect(screen.getByText('JSON Data')).toBeInTheDocument();
    expect(screen.getByText('CSV Export')).toBeInTheDocument();
    expect(screen.getByText('Excel File')).toBeInTheDocument();
  });

  it('handles PDF export', async () => {
    render(<ExportHub {...mockProps} />);
    
    const pdfButton = screen.getByRole('button', { name: /export pdf/i });
    fireEvent.click(pdfButton);
    
    await waitFor(() => {
      expect(mockProps.onExportComplete).toHaveBeenCalledWith(
        'pdf',
        expect.stringContaining('Uveddi_Dashboard_Report.pdf')
      );
    });
  });

  it('handles PNG export', async () => {
    render(<ExportHub {...mockProps} />);
    
    const pngButton = screen.getByRole('button', { name: /export png/i });
    fireEvent.click(pngButton);
    
    await waitFor(() => {
      expect(mockProps.onExportComplete).toHaveBeenCalledWith(
        'png',
        expect.stringContaining('.png')
      );
    });
  });

  it('handles JSON data export', async () => {
    render(<ExportHub {...mockProps} />);
    
    const jsonButton = screen.getByRole('button', { name: /export json/i });
    fireEvent.click(jsonButton);
    
    await waitFor(() => {
      expect(mockProps.onExportComplete).toHaveBeenCalledWith(
        'json',
        expect.stringContaining('.json')
      );
    });
  });

  it('shows export options dialog', async () => {
    render(<ExportHub {...mockProps} />);
    
    const pdfButton = screen.getByRole('button', { name: /export pdf/i });
    fireEvent.click(pdfButton);
    
    await waitFor(() => {
      expect(screen.getByText('Export Options')).toBeInTheDocument();
      expect(screen.getByText('Include Charts')).toBeInTheDocument();
      expect(screen.getByText('Include Data Tables')).toBeInTheDocument();
    });
  });

  it('handles custom export options', async () => {
    render(<ExportHub {...mockProps} />);
    
    const pdfButton = screen.getByRole('button', { name: /export pdf/i });
    fireEvent.click(pdfButton);
    
    // Toggle options
    const includeChartsToggle = screen.getByRole('switch', { name: /include charts/i });
    fireEvent.click(includeChartsToggle);
    
    const exportButton = screen.getByRole('button', { name: /export/i });
    fireEvent.click(exportButton);
    
    await waitFor(() => {
      expect(mockProps.onExportComplete).toHaveBeenCalled();
    });
  });

  it('displays export progress', async () => {
    render(<ExportHub {...mockProps} />);
    
    const pdfButton = screen.getByRole('button', { name: /export pdf/i });
    fireEvent.click(pdfButton);
    
    // Should show progress indicator
    expect(screen.getByRole('progressbar')).toBeInTheDocument();
    expect(screen.getByText(/Generating PDF/)).toBeInTheDocument();
  });

  it('shows file size estimation', () => {
    render(<ExportHub {...mockProps} />);
    
    expect(screen.getByText(/~2.5 MB/)).toBeInTheDocument(); // PDF estimate
    expect(screen.getByText(/~500 KB/)).toBeInTheDocument(); // PNG estimate
  });

  it('handles batch export', async () => {
    render(<ExportHub {...mockProps} />);
    
    const batchButton = screen.getByRole('button', { name: /batch export/i });
    fireEvent.click(batchButton);
    
    // Select multiple formats
    const pdfCheckbox = screen.getByRole('checkbox', { name: /pdf/i });
    const pngCheckbox = screen.getByRole('checkbox', { name: /png/i });
    
    fireEvent.click(pdfCheckbox);
    fireEvent.click(pngCheckbox);
    
    const exportAllButton = screen.getByRole('button', { name: /export all/i });
    fireEvent.click(exportAllButton);
    
    await waitFor(() => {
      expect(mockProps.onExportComplete).toHaveBeenCalledTimes(2);
    });
  });

  it('displays export history', () => {
    const propsWithHistory = {
      ...mockProps,
      exportHistory: [
        {
          id: '1',
          format: 'pdf',
          filename: 'report-2024-01-15.pdf',
          size: '2.3 MB',
          timestamp: '2024-01-15T10:30:00Z'
        }
      ]
    };
    
    render(<ExportHub {...propsWithHistory} />);
    
    const historyTab = screen.getByRole('tab', { name: /history/i });
    fireEvent.click(historyTab);
    
    expect(screen.getByText('report-2024-01-15.pdf')).toBeInTheDocument();
    expect(screen.getByText('2.3 MB')).toBeInTheDocument();
  });

  it('handles scheduled exports', async () => {
    render(<ExportHub {...mockProps} />);
    
    const scheduleButton = screen.getByRole('button', { name: /schedule export/i });
    fireEvent.click(scheduleButton);
    
    const frequencySelect = screen.getByRole('combobox', { name: /frequency/i });
    fireEvent.mouseDown(frequencySelect);
    
    const weeklyOption = screen.getByText('Weekly');
    fireEvent.click(weeklyOption);
    
    const saveScheduleButton = screen.getByRole('button', { name: /save schedule/i });
    fireEvent.click(saveScheduleButton);
    
    await waitFor(() => {
      expect(screen.getByText(/Schedule saved/)).toBeInTheDocument();
    });
  });

  it('shows export templates', () => {
    render(<ExportHub {...mockProps} />);
    
    const templatesTab = screen.getByRole('tab', { name: /templates/i });
    fireEvent.click(templatesTab);
    
    expect(screen.getByText('Executive Summary')).toBeInTheDocument();
    expect(screen.getByText('Technical Deep Dive')).toBeInTheDocument();
    expect(screen.getByText('Quick Overview')).toBeInTheDocument();
  });

  it('handles custom template creation', async () => {
    render(<ExportHub {...mockProps} />);
    
    const templatesTab = screen.getByRole('tab', { name: /templates/i });
    fireEvent.click(templatesTab);
    
    const createTemplateButton = screen.getByRole('button', { name: /create template/i });
    fireEvent.click(createTemplateButton);
    
    const templateNameInput = screen.getByPlaceholderText(/Template name/);
    fireEvent.change(templateNameInput, { target: { value: 'My Custom Template' } });
    
    const saveButton = screen.getByRole('button', { name: /save template/i });
    fireEvent.click(saveButton);
    
    await waitFor(() => {
      expect(screen.getByText('My Custom Template')).toBeInTheDocument();
    });
  });

  it('displays export error handling', async () => {
    // Mock export failure
    jest.spyOn(console, 'error').mockImplementation(() => {});
    
    render(<ExportHub {...mockProps} />);
    
    const pdfButton = screen.getByRole('button', { name: /export pdf/i });
    fireEvent.click(pdfButton);
    
    // Simulate export error
    await waitFor(() => {
      if (mockProps.onExportError.mock.calls.length === 0) {
        mockProps.onExportError('Export failed: Network error');
      }
    });
    
    expect(screen.getByText(/Export failed/)).toBeInTheDocument();
  });

  it('shows file format descriptions', () => {
    render(<ExportHub {...mockProps} />);
    
    expect(screen.getByText(/Best for sharing and printing/)).toBeInTheDocument(); // PDF
    expect(screen.getByText(/High quality image/)).toBeInTheDocument(); // PNG
    expect(screen.getByText(/Raw data for analysis/)).toBeInTheDocument(); // JSON
  });

  it('handles cloud storage integration', async () => {
    const propsWithCloud = {
      ...mockProps,
      cloudStorageEnabled: true
    };
    
    render(<ExportHub {...propsWithCloud} />);
    
    const cloudButton = screen.getByRole('button', { name: /save to cloud/i });
    fireEvent.click(cloudButton);
    
    expect(screen.getByText('Google Drive')).toBeInTheDocument();
    expect(screen.getByText('Dropbox')).toBeInTheDocument();
    expect(screen.getByText('OneDrive')).toBeInTheDocument();
  });

  it('displays export quotas and limits', () => {
    const propsWithLimits = {
      ...mockProps,
      exportLimits: {
        daily: 50,
        used: 12,
        premium: false
      }
    };
    
    render(<ExportHub {...propsWithLimits} />);
    
    expect(screen.getByText(/12 of 50 exports used today/)).toBeInTheDocument();
    expect(screen.getByText(/Upgrade to Premium/)).toBeInTheDocument();
  });

  it('has proper accessibility attributes', () => {
    render(<ExportHub {...mockProps} />);
    
    const exportButtons = screen.getAllByRole('button', { name: /export/i });
    exportButtons.forEach(button => {
      expect(button).toHaveAttribute('aria-label');
    });
    
    const tabList = screen.getByRole('tablist');
    expect(tabList).toHaveAttribute('aria-label');
    
    const progressBar = screen.queryByRole('progressbar');
    if (progressBar) {
      expect(progressBar).toHaveAttribute('aria-label');
    }
  });

  it('handles keyboard navigation', async () => {
    render(<ExportHub {...mockProps} />);
    
    const firstButton = screen.getAllByRole('button')[0];
    firstButton.focus();
    
    expect(document.activeElement).toBe(firstButton);
    
    // Tab navigation
    fireEvent.keyDown(firstButton, { key: 'Tab' });
  });

  it('shows download progress for large files', async () => {
    render(<ExportHub {...mockProps} />);
    
    const pdfButton = screen.getByRole('button', { name: /export pdf/i });
    fireEvent.click(pdfButton);
    
    await waitFor(() => {
      const progress = screen.getByRole('progressbar');
      expect(progress).toHaveAttribute('aria-valuenow');
      expect(progress).toHaveAttribute('aria-valuemin', '0');
      expect(progress).toHaveAttribute('aria-valuemax', '100');
    });
  });
});