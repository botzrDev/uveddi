import React, { useMemo, useCallback, useState, useRef } from 'react';
import {
  Box,
  Card,
  CardContent,
  Typography,
  IconButton,
  Tooltip,
  Button,
  TextField,
  Avatar,
  Chip,
  Paper,
  Grid,
  List,
  ListItem,
  ListItemIcon,
  ListItemText,
  ListItemSecondaryAction,
  Divider,
  Badge,
  Menu,
  MenuItem,
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  FormControl,
  FormLabel,
  Select,
  Autocomplete,
  Alert,
  Tabs,
  Tab,
  useTheme,
  Collapse,
  InputAdornment,
} from '@mui/material';
import {
  CommentOutlined,
  PersonAddOutlined,
  AssignmentIndOutlined,
  NotificationsOutlined,
  SendOutlined,
  ReplyOutlined,
  EditOutlined,
  DeleteOutlined,
  CheckCircleOutlined,
  ErrorOutlined,
  InfoOutlined,
  QuestionAnswerOutlined,
  LightbulbOutlined,
  ThumbUpOutlined,
  ThumbDownOutlined,
  MoreVertOutlined,
  PushPinOutlined,
  GroupOutlined,
  TaskAltOutlined,
  FlagOutlined,
  CalendarTodayOutlined,
  AccessTimeOutlined,
  PriorityHighOutlined,
  ExpandMoreOutlined,
  ExpandLessOutlined,
  AlternateEmailOutlined,
  EmojiEmotionsOutlined,
} from '@mui/icons-material';
import { formatDistanceToNow, parseISO } from 'date-fns';

import type { Finding } from '../../types/api';
import type {
  Annotation,
  AnnotationReply,
  User,
  Review,
  ResolutionInfo,
  ResponsiveComponentProps,
} from '../../types/dashboard';

import { dashboardTheme } from '../../utils/dashboardTheme';

interface CollaborationPanelProps extends ResponsiveComponentProps {
  findings: Finding[];
  currentUser?: User;
  onAnnotationAdd?: (annotation: Omit<Annotation, 'id' | 'timestamp'>) => void;
  onAnnotationReply?: (annotationId: string, reply: Omit<AnnotationReply, 'id' | 'timestamp'>) => void;
  onReviewAssign?: (review: Omit<Review, 'id'>) => void;
  onReviewStatusChange?: (reviewId: string, status: Review['status']) => void;
  teamMembers?: User[];
  height?: number;
}

// Mock data for demonstration
const mockCurrentUser: User = {
  id: 'user1',
  name: 'John Smith',
  email: 'john.smith@company.com',
  avatar: '/avatars/john.jpg',
  role: 'developer',
};

const mockTeamMembers: User[] = [
  {
    id: 'user2',
    name: 'Sarah Johnson',
    email: 'sarah.johnson@company.com',
    avatar: '/avatars/sarah.jpg',
    role: 'lead',
  },
  {
    id: 'user3',
    name: 'Mike Chen',
    email: 'mike.chen@company.com',
    avatar: '/avatars/mike.jpg',
    role: 'architect',
  },
  {
    id: 'user4',
    name: 'Emily Rodriguez',
    email: 'emily.rodriguez@company.com',
    avatar: '/avatars/emily.jpg',
    role: 'manager',
  },
];

const mockAnnotations: Annotation[] = [
  {
    id: 'ann1',
    findingId: 'finding1',
    author: mockTeamMembers[0],
    content: 'This looks like a critical issue that needs immediate attention. The god object pattern is affecting multiple modules.',
    type: 'comment',
    timestamp: new Date(Date.now() - 3600000).toISOString(), // 1 hour ago
    replies: [
      {
        id: 'reply1',
        author: mockCurrentUser,
        content: 'I agree. Should we prioritize this for the next sprint?',
        timestamp: new Date(Date.now() - 1800000).toISOString(), // 30 minutes ago
      },
    ],
    mentions: [mockCurrentUser],
    tags: ['critical', 'architecture'],
  },
  {
    id: 'ann2',
    findingId: 'finding2',
    author: mockTeamMembers[1],
    content: 'I suggest using the Strategy pattern here instead of multiple if-else statements.',
    type: 'suggestion',
    timestamp: new Date(Date.now() - 7200000).toISOString(), // 2 hours ago
    replies: [],
    mentions: [],
    tags: ['refactoring', 'pattern'],
  },
  {
    id: 'ann3',
    findingId: 'finding3',
    author: mockTeamMembers[2],
    content: 'Can we get more context on the business requirements for this component?',
    type: 'question',
    timestamp: new Date(Date.now() - 10800000).toISOString(), // 3 hours ago
    replies: [],
    mentions: [mockTeamMembers[0]],
    tags: ['requirements'],
  },
];

const mockReviews: Review[] = [
  {
    id: 'rev1',
    title: 'Critical Security Issues Review',
    description: 'Review all critical security findings before next release',
    assignee: mockCurrentUser,
    reviewer: mockTeamMembers[0],
    status: 'in_progress',
    priority: 'high',
    dueDate: new Date(Date.now() + 86400000 * 2).toISOString(), // 2 days from now
    findings: ['finding1', 'finding3'],
    annotations: ['ann1'],
  },
  {
    id: 'rev2',
    title: 'Architecture Improvements',
    description: 'Evaluate and plan refactoring for identified architectural issues',
    assignee: mockTeamMembers[1],
    reviewer: mockTeamMembers[2],
    status: 'pending',
    priority: 'medium',
    findings: ['finding2'],
    annotations: ['ann2'],
  },
];

const CollaborationPanel: React.FC<CollaborationPanelProps> = ({
  findings,
  currentUser = mockCurrentUser,
  onAnnotationAdd,
  onAnnotationReply,
  onReviewAssign,
  onReviewStatusChange,
  teamMembers = mockTeamMembers,
  height = 600,
  loading = false,
  error,
  className,
  testId,
}) => {
  const theme = useTheme();
  const [activeTab, setActiveTab] = useState(0);
  const [annotations, setAnnotations] = useState<Annotation[]>(mockAnnotations);
  const [reviews, setReviews] = useState<Review[]>(mockReviews);
  
  // Dialog states
  const [annotationDialogOpen, setAnnotationDialogOpen] = useState(false);
  const [reviewDialogOpen, setReviewDialogOpen] = useState(false);
  const [selectedFinding, setSelectedFinding] = useState<Finding | null>(null);
  
  // Form states
  const [newAnnotation, setNewAnnotation] = useState({
    content: '',
    type: 'comment' as Annotation['type'],
    mentions: [] as User[],
    tags: [] as string[],
  });
  const [newReview, setNewReview] = useState({
    title: '',
    description: '',
    assignee: null as User | null,
    priority: 'medium' as Review['priority'],
    dueDate: '',
    findings: [] as string[],
  });
  
  // UI states
  const [expandedAnnotations, setExpandedAnnotations] = useState<string[]>([]);
  const [replyingTo, setReplyingTo] = useState<string | null>(null);
  const [replyContent, setReplyContent] = useState('');
  const [anchorEl, setAnchorEl] = useState<null | HTMLElement>(null);
  const [selectedAnnotation, setSelectedAnnotation] = useState<string | null>(null);

  // Notification count
  const notificationCount = useMemo(() => {
    return annotations.filter(annotation => 
      annotation.mentions.some(user => user.id === currentUser.id) &&
      new Date(annotation.timestamp) > new Date(Date.now() - 86400000) // Last 24 hours
    ).length;
  }, [annotations, currentUser]);

  // Get annotations for specific finding
  const getAnnotationsForFinding = useCallback((findingId: string) => {
    return annotations.filter(annotation => annotation.findingId === findingId);
  }, [annotations]);

  // Get reviews for current user
  const getUserReviews = useCallback((userId: string) => {
    return reviews.filter(review => 
      review.assignee.id === userId || review.reviewer.id === userId
    );
  }, [reviews]);

  // Handle annotation submission
  const handleAnnotationSubmit = useCallback(() => {
    if (!selectedFinding || !newAnnotation.content.trim()) return;

    const annotation: Annotation = {
      id: `ann_${Date.now()}`,
      findingId: selectedFinding.id,
      author: currentUser,
      content: newAnnotation.content,
      type: newAnnotation.type,
      timestamp: new Date().toISOString(),
      replies: [],
      mentions: newAnnotation.mentions,
      tags: newAnnotation.tags,
    };

    setAnnotations(prev => [annotation, ...prev]);
    
    if (onAnnotationAdd) {
      onAnnotationAdd({
        findingId: annotation.findingId,
        author: annotation.author,
        content: annotation.content,
        type: annotation.type,
        replies: [],
        mentions: annotation.mentions,
        tags: annotation.tags,
      });
    }

    // Reset form
    setNewAnnotation({
      content: '',
      type: 'comment',
      mentions: [],
      tags: [],
    });
    setAnnotationDialogOpen(false);
    setSelectedFinding(null);
  }, [selectedFinding, newAnnotation, currentUser, onAnnotationAdd]);

  // Handle reply submission
  const handleReplySubmit = useCallback((annotationId: string) => {
    if (!replyContent.trim()) return;

    const reply: AnnotationReply = {
      id: `reply_${Date.now()}`,
      author: currentUser,
      content: replyContent,
      timestamp: new Date().toISOString(),
    };

    setAnnotations(prev => prev.map(annotation => 
      annotation.id === annotationId
        ? { ...annotation, replies: [...(annotation.replies || []), reply] }
        : annotation
    ));

    if (onAnnotationReply) {
      onAnnotationReply(annotationId, {
        author: reply.author,
        content: reply.content,
      });
    }

    setReplyContent('');
    setReplyingTo(null);
  }, [replyContent, currentUser, onAnnotationReply]);

  // Handle review submission
  const handleReviewSubmit = useCallback(() => {
    if (!newReview.title.trim() || !newReview.assignee) return;

    const review: Review = {
      id: `rev_${Date.now()}`,
      title: newReview.title,
      description: newReview.description,
      assignee: newReview.assignee,
      reviewer: currentUser,
      status: 'pending',
      priority: newReview.priority,
      dueDate: newReview.dueDate,
      findings: newReview.findings,
      annotations: [],
    };

    setReviews(prev => [review, ...prev]);

    if (onReviewAssign) {
      onReviewAssign(review);
    }

    // Reset form
    setNewReview({
      title: '',
      description: '',
      assignee: null,
      priority: 'medium',
      dueDate: '',
      findings: [],
    });
    setReviewDialogOpen(false);
  }, [newReview, currentUser, onReviewAssign]);

  // Handle annotation expansion
  const handleAnnotationToggle = useCallback((annotationId: string) => {
    setExpandedAnnotations(prev => 
      prev.includes(annotationId)
        ? prev.filter(id => id !== annotationId)
        : [...prev, annotationId]
    );
  }, []);

  // Get annotation type icon and color
  const getAnnotationTypeInfo = useCallback((type: Annotation['type']) => {
    switch (type) {
      case 'comment':
        return { icon: <CommentOutlined />, color: 'primary', label: 'Comment' };
      case 'question':
        return { icon: <QuestionAnswerOutlined />, color: 'info', label: 'Question' };
      case 'suggestion':
        return { icon: <LightbulbOutlined />, color: 'warning', label: 'Suggestion' };
      case 'approval':
        return { icon: <CheckCircleOutlined />, color: 'success', label: 'Approval' };
      default:
        return { icon: <CommentOutlined />, color: 'default', label: 'Comment' };
    }
  }, []);

  // Get review priority color
  const getReviewPriorityColor = useCallback((priority: Review['priority']) => {
    switch (priority) {
      case 'critical':
        return 'error';
      case 'high':
        return 'warning';
      case 'medium':
        return 'info';
      case 'low':
        return 'success';
      default:
        return 'default';
    }
  }, []);

  // Get review status color
  const getReviewStatusColor = useCallback((status: Review['status']) => {
    switch (status) {
      case 'completed':
        return 'success';
      case 'in_progress':
        return 'warning';
      case 'dismissed':
        return 'error';
      default:
        return 'default';
    }
  }, []);

  if (loading) {
    return (
      <Card className={className} data-testid={testId}>
        <CardContent>
          <Box display="flex" alignItems="center" justifyContent="center" height={height}>
            <Typography variant="body2" color="text.secondary">
              Loading collaboration panel...
            </Typography>
          </Box>
        </CardContent>
      </Card>
    );
  }

  if (error) {
    return (
      <Card className={className} data-testid={testId}>
        <CardContent>
          <Box display="flex" alignItems="center" justifyContent="center" height={height}>
            <Alert severity="error" sx={{ maxWidth: 400 }}>
              <Typography variant="body2">
                Error loading collaboration: {error}
              </Typography>
            </Alert>
          </Box>
        </CardContent>
      </Card>
    );
  }

  return (
    <Box className={className} data-testid={testId}>
      {/* Header */}
      <Box display="flex" alignItems="center" justifyContent="space-between" mb={2}>
        <Box>
          <Typography variant="h5" component="h2" fontWeight="600" color="text.primary">
            Team Collaboration
          </Typography>
          <Typography variant="body2" color="text.secondary">
            Discuss findings and coordinate reviews
          </Typography>
        </Box>
        
        <Box display="flex" alignItems="center" gap={1}>
          <Badge badgeContent={notificationCount} color="error">
            <IconButton>
              <NotificationsOutlined />
            </IconButton>
          </Badge>
          
          <Tooltip title="Team collaboration and code review features" arrow>
            <IconButton>
              <InfoOutlined />
            </IconButton>
          </Tooltip>
        </Box>
      </Box>

      <Card>
        {/* Tabs */}
        <Box sx={{ borderBottom: 1, borderColor: 'divider' }}>
          <Tabs value={activeTab} onChange={(_, newValue) => setActiveTab(newValue)}>
            <Tab 
              label={
                <Box display="flex" alignItems="center" gap={0.5}>
                  <CommentOutlined fontSize="small" />
                  Discussions ({annotations.length})
                </Box>
              }
            />
            <Tab 
              label={
                <Box display="flex" alignItems="center" gap={0.5}>
                  <AssignmentIndOutlined fontSize="small" />
                  Reviews ({reviews.length})
                </Box>
              }
            />
            <Tab 
              label={
                <Box display="flex" alignItems="center" gap={0.5}>
                  <GroupOutlined fontSize="small" />
                  Team ({teamMembers.length})
                </Box>
              }
            />
          </Tabs>
        </Box>

        <CardContent sx={{ height: height - 120, overflow: 'auto' }}>
          {/* Discussions Tab */}
          {activeTab === 0 && (
            <Box>
              {/* Add annotation button */}
              <Box display="flex" gap={1} mb={3}>
                <Button
                  variant="contained"
                  startIcon={<CommentOutlined />}
                  onClick={() => setAnnotationDialogOpen(true)}
                >
                  Add Discussion
                </Button>
                <Button
                  variant="outlined"
                  startIcon={<PersonAddOutlined />}
                  onClick={() => {
                    // Mock mention functionality
                    alert('Mention functionality would open user selector');
                  }}
                >
                  Mention
                </Button>
              </Box>

              {/* Annotations list */}
              {annotations.length === 0 ? (
                <Box display="flex" flexDirection="column" alignItems="center" py={4}>
                  <CommentOutlined sx={{ fontSize: 64, color: 'text.secondary', mb: 2 }} />
                  <Typography variant="h6" color="text.secondary" mb={1}>
                    No Discussions Yet
                  </Typography>
                  <Typography variant="body2" color="text.secondary" textAlign="center">
                    Start a discussion about findings to collaborate with your team.
                  </Typography>
                </Box>
              ) : (
                <List>
                  {annotations.map((annotation, index) => {
                    const typeInfo = getAnnotationTypeInfo(annotation.type);
                    const isExpanded = expandedAnnotations.includes(annotation.id);
                    const finding = findings.find(f => f.id === annotation.findingId);
                    
                    return (
                      <React.Fragment key={annotation.id}>
                        <ListItem
                          alignItems="flex-start"
                          sx={{
                            bgcolor: 'background.paper',
                            borderRadius: 1,
                            mb: 1,
                            border: '1px solid',
                            borderColor: 'divider',
                          }}
                        >
                          <ListItemIcon>
                            <Avatar
                              src={annotation.author.avatar}
                              sx={{
                                bgcolor: `${typeInfo.color}.light`,
                                color: `${typeInfo.color}.main`,
                                width: 40,
                                height: 40,
                              }}
                            >
                              {typeInfo.icon}
                            </Avatar>
                          </ListItemIcon>
                          <ListItemText
                            primary={
                              <Box>
                                <Box display="flex" alignItems="center" gap={1} mb={0.5}>
                                  <Typography variant="subtitle2" fontWeight="600">
                                    {annotation.author.name}
                                  </Typography>
                                  <Chip
                                    label={typeInfo.label}
                                    size="small"
                                    color={typeInfo.color as any}
                                    variant="outlined"
                                  />
                                  <Typography variant="caption" color="text.secondary">
                                    {formatDistanceToNow(parseISO(annotation.timestamp))} ago
                                  </Typography>
                                </Box>
                                
                                {finding && (
                                  <Alert severity="info" sx={{ mb: 1 }}>
                                    <Typography variant="caption">
                                      <strong>Related to:</strong> {finding.title}
                                    </Typography>
                                  </Alert>
                                )}
                                
                                <Typography variant="body2" sx={{ mb: 1 }}>
                                  {annotation.content}
                                </Typography>
                                
                                {/* Tags and mentions */}
                                <Box display="flex" flexWrap="wrap" gap={0.5} mb={1}>
                                  {annotation.tags.map(tag => (
                                    <Chip
                                      key={tag}
                                      label={tag}
                                      size="small"
                                      variant="outlined"
                                      sx={{ height: 20, fontSize: '0.6rem' }}
                                    />
                                  ))}
                                  {annotation.mentions.map(user => (
                                    <Chip
                                      key={user.id}
                                      label={`@${user.name}`}
                                      size="small"
                                      color="primary"
                                      variant="outlined"
                                      sx={{ height: 20, fontSize: '0.6rem' }}
                                    />
                                  ))}
                                </Box>
                                
                                {/* Action buttons */}
                                <Box display="flex" alignItems="center" gap={1}>
                                  <Button
                                    size="small"
                                    startIcon={<ReplyOutlined />}
                                    onClick={() => setReplyingTo(annotation.id)}
                                  >
                                    Reply
                                  </Button>
                                  <Button
                                    size="small"
                                    startIcon={<ThumbUpOutlined />}
                                    color="success"
                                  >
                                    Like
                                  </Button>
                                  {(annotation.replies?.length || 0) > 0 && (
                                    <Button
                                      size="small"
                                      startIcon={isExpanded ? <ExpandLessOutlined /> : <ExpandMoreOutlined />}
                                      onClick={() => handleAnnotationToggle(annotation.id)}
                                    >
                                      {annotation.replies?.length} replies
                                    </Button>
                                  )}
                                </Box>
                              </Box>
                            }
                          />
                          <ListItemSecondaryAction>
                            <IconButton
                              edge="end"
                              onClick={(e) => {
                                setAnchorEl(e.currentTarget);
                                setSelectedAnnotation(annotation.id);
                              }}
                            >
                              <MoreVertOutlined />
                            </IconButton>
                          </ListItemSecondaryAction>
                        </ListItem>

                        {/* Replies */}
                        <Collapse in={isExpanded}>
                          <Box ml={7} mb={2}>
                            {annotation.replies?.map(reply => (
                              <Paper
                                key={reply.id}
                                sx={{ p: 2, mb: 1, bgcolor: 'action.hover' }}
                              >
                                <Box display="flex" alignItems="center" gap={1} mb={1}>
                                  <Avatar
                                    src={reply.author.avatar}
                                    sx={{ width: 24, height: 24 }}
                                  >
                                    {reply.author.name[0]}
                                  </Avatar>
                                  <Typography variant="body2" fontWeight="600">
                                    {reply.author.name}
                                  </Typography>
                                  <Typography variant="caption" color="text.secondary">
                                    {formatDistanceToNow(parseISO(reply.timestamp))} ago
                                  </Typography>
                                </Box>
                                <Typography variant="body2">
                                  {reply.content}
                                </Typography>
                              </Paper>
                            ))}
                          </Box>
                        </Collapse>

                        {/* Reply input */}
                        <Collapse in={replyingTo === annotation.id}>
                          <Box ml={7} mb={2}>
                            <Paper sx={{ p: 2 }}>
                              <TextField
                                fullWidth
                                multiline
                                rows={2}
                                value={replyContent}
                                onChange={(e) => setReplyContent(e.target.value)}
                                placeholder="Write a reply..."
                                variant="outlined"
                                size="small"
                                sx={{ mb: 1 }}
                              />
                              <Box display="flex" justifyContent="flex-end" gap={1}>
                                <Button
                                  size="small"
                                  onClick={() => setReplyingTo(null)}
                                >
                                  Cancel
                                </Button>
                                <Button
                                  size="small"
                                  variant="contained"
                                  startIcon={<SendOutlined />}
                                  onClick={() => handleReplySubmit(annotation.id)}
                                  disabled={!replyContent.trim()}
                                >
                                  Reply
                                </Button>
                              </Box>
                            </Paper>
                          </Box>
                        </Collapse>

                        {index < annotations.length - 1 && <Divider sx={{ my: 1 }} />}
                      </React.Fragment>
                    );
                  })}
                </List>
              )}
            </Box>
          )}

          {/* Reviews Tab */}
          {activeTab === 1 && (
            <Box>
              {/* Add review button */}
              <Box display="flex" gap={1} mb={3}>
                <Button
                  variant="contained"
                  startIcon={<AssignmentIndOutlined />}
                  onClick={() => setReviewDialogOpen(true)}
                >
                  Create Review
                </Button>
              </Box>

              {/* Reviews list */}
              {reviews.length === 0 ? (
                <Box display="flex" flexDirection="column" alignItems="center" py={4}>
                  <AssignmentIndOutlined sx={{ fontSize: 64, color: 'text.secondary', mb: 2 }} />
                  <Typography variant="h6" color="text.secondary" mb={1}>
                    No Reviews Yet
                  </Typography>
                  <Typography variant="body2" color="text.secondary" textAlign="center">
                    Create reviews to track progress on findings and coordinate team efforts.
                  </Typography>
                </Box>
              ) : (
                <List>
                  {reviews.map((review, index) => (
                    <React.Fragment key={review.id}>
                      <ListItem
                        alignItems="flex-start"
                        sx={{
                          bgcolor: 'background.paper',
                          borderRadius: 1,
                          mb: 1,
                          border: '1px solid',
                          borderColor: 'divider',
                          borderLeft: `4px solid ${theme.palette[getReviewPriorityColor(review.priority) as keyof typeof theme.palette].main}`,
                        }}
                      >
                        <ListItemIcon>
                          <Avatar
                            sx={{
                              bgcolor: `${getReviewStatusColor(review.status)}.light`,
                              color: `${getReviewStatusColor(review.status)}.main`,
                              width: 40,
                              height: 40,
                            }}
                          >
                            <TaskAltOutlined />
                          </Avatar>
                        </ListItemIcon>
                        <ListItemText
                          primary={
                            <Box>
                              <Box display="flex" alignItems="center" gap={1} mb={1}>
                                <Typography variant="subtitle1" fontWeight="600">
                                  {review.title}
                                </Typography>
                                <Chip
                                  label={review.status.replace('_', ' ')}
                                  size="small"
                                  color={getReviewStatusColor(review.status) as any}
                                  variant="outlined"
                                />
                                <Chip
                                  label={review.priority}
                                  size="small"
                                  color={getReviewPriorityColor(review.priority) as any}
                                />
                              </Box>
                              
                              <Typography variant="body2" color="text.secondary" sx={{ mb: 2 }}>
                                {review.description}
                              </Typography>
                              
                              <Grid container spacing={2} sx={{ mb: 2 }}>
                                <Grid item xs={12} sm={6}>
                                  <Box display="flex" alignItems="center" gap={1}>
                                    <Typography variant="caption" color="text.secondary">
                                      Assignee:
                                    </Typography>
                                    <Avatar
                                      src={review.assignee.avatar}
                                      sx={{ width: 20, height: 20 }}
                                    >
                                      {review.assignee.name[0]}
                                    </Avatar>
                                    <Typography variant="body2">
                                      {review.assignee.name}
                                    </Typography>
                                  </Box>
                                </Grid>
                                <Grid item xs={12} sm={6}>
                                  <Box display="flex" alignItems="center" gap={1}>
                                    <Typography variant="caption" color="text.secondary">
                                      Reviewer:
                                    </Typography>
                                    <Avatar
                                      src={review.reviewer.avatar}
                                      sx={{ width: 20, height: 20 }}
                                    >
                                      {review.reviewer.name[0]}
                                    </Avatar>
                                    <Typography variant="body2">
                                      {review.reviewer.name}
                                    </Typography>
                                  </Box>
                                </Grid>
                              </Grid>
                              
                              {review.dueDate && (
                                <Box display="flex" alignItems="center" gap={1} mb={1}>
                                  <CalendarTodayOutlined fontSize="small" color="action" />
                                  <Typography variant="caption" color="text.secondary">
                                    Due: {format(parseISO(review.dueDate), 'MMM dd, yyyy')}
                                  </Typography>
                                </Box>
                              )}
                              
                              <Box display="flex" alignItems="center" gap={1} mb={2}>
                                <Typography variant="caption" color="text.secondary">
                                  Findings: {review.findings.length}
                                </Typography>
                                <Typography variant="caption" color="text.secondary">
                                  • Discussions: {review.annotations.length}
                                </Typography>
                              </Box>
                              
                              {/* Action buttons */}
                              <Box display="flex" gap={1}>
                                {review.status !== 'completed' && (
                                  <Button
                                    size="small"
                                    variant="contained"
                                    startIcon={<CheckCircleOutlined />}
                                    onClick={() => {
                                      const newStatus = review.status === 'pending' ? 'in_progress' : 'completed';
                                      setReviews(prev => prev.map(r => 
                                        r.id === review.id ? { ...r, status: newStatus } : r
                                      ));
                                      if (onReviewStatusChange) {
                                        onReviewStatusChange(review.id, newStatus);
                                      }
                                    }}
                                  >
                                    {review.status === 'pending' ? 'Start Review' : 'Complete'}
                                  </Button>
                                )}
                                <Button
                                  size="small"
                                  startIcon={<CommentOutlined />}
                                  onClick={() => {
                                    setActiveTab(0); // Switch to discussions tab
                                  }}
                                >
                                  Discuss
                                </Button>
                              </Box>
                            </Box>
                          }
                        />
                      </ListItem>
                      {index < reviews.length - 1 && <Divider sx={{ my: 1 }} />}
                    </React.Fragment>
                  ))}
                </List>
              )}
            </Box>
          )}

          {/* Team Tab */}
          {activeTab === 2 && (
            <Box>
              <Typography variant="h6" fontWeight="600" mb={2}>
                Team Members ({teamMembers.length})
              </Typography>
              
              <Grid container spacing={2}>
                {teamMembers.map(member => {
                  const memberReviews = getUserReviews(member.id);
                  const memberAnnotations = annotations.filter(a => a.author.id === member.id);
                  
                  return (
                    <Grid item xs={12} sm={6} md={4} key={member.id}>
                      <Card>
                        <CardContent>
                          <Box display="flex" alignItems="center" gap={2} mb={2}>
                            <Avatar
                              src={member.avatar}
                              sx={{ width: 48, height: 48 }}
                            >
                              {member.name[0]}
                            </Avatar>
                            <Box flex={1}>
                              <Typography variant="subtitle1" fontWeight="600">
                                {member.name}
                              </Typography>
                              <Typography variant="body2" color="text.secondary">
                                {member.email}
                              </Typography>
                              <Chip
                                label={member.role}
                                size="small"
                                color="primary"
                                variant="outlined"
                                sx={{ mt: 0.5 }}
                              />
                            </Box>
                          </Box>
                          
                          <Box display="flex" justifyContent="space-between" mb={2}>
                            <Box textAlign="center">
                              <Typography variant="h6" fontWeight="700" color="primary">
                                {memberReviews.length}
                              </Typography>
                              <Typography variant="caption" color="text.secondary">
                                Reviews
                              </Typography>
                            </Box>
                            <Box textAlign="center">
                              <Typography variant="h6" fontWeight="700" color="secondary">
                                {memberAnnotations.length}
                              </Typography>
                              <Typography variant="caption" color="text.secondary">
                                Comments
                              </Typography>
                            </Box>
                          </Box>
                          
                          <Button
                            fullWidth
                            variant="outlined"
                            startIcon={<AlternateEmailOutlined />}
                            onClick={() => {
                              // Mock functionality - would normally open chat or mention dialog
                              alert(`Mention ${member.name} functionality`);
                            }}
                          >
                            Mention
                          </Button>
                        </CardContent>
                      </Card>
                    </Grid>
                  );
                })}
              </Grid>
            </Box>
          )}
        </CardContent>
      </Card>

      {/* Add Annotation Dialog */}
      <Dialog
        open={annotationDialogOpen}
        onClose={() => setAnnotationDialogOpen(false)}
        maxWidth="md"
        fullWidth
      >
        <DialogTitle>Add Discussion</DialogTitle>
        <DialogContent>
          <Box display="flex" flexDirection="column" gap={2} pt={1}>
            {/* Finding selector */}
            <FormControl fullWidth>
              <FormLabel>Select Finding</FormLabel>
              <Autocomplete
                options={findings}
                getOptionLabel={(option) => option.title}
                value={selectedFinding}
                onChange={(_, value) => setSelectedFinding(value)}
                renderInput={(params) => (
                  <TextField {...params} placeholder="Choose a finding to discuss..." />
                )}
                renderOption={(props, option) => (
                  <Box component="li" {...props}>
                    <Box>
                      <Typography variant="body2" fontWeight="600">
                        {option.title}
                      </Typography>
                      <Typography variant="caption" color="text.secondary">
                        {option.file} • {option.severity}
                      </Typography>
                    </Box>
                  </Box>
                )}
              />
            </FormControl>

            {/* Content */}
            <TextField
              fullWidth
              multiline
              rows={4}
              label="Message"
              value={newAnnotation.content}
              onChange={(e) => setNewAnnotation(prev => ({ ...prev, content: e.target.value }))}
              placeholder="Share your thoughts, ask questions, or suggest improvements..."
            />

            {/* Type selector */}
            <FormControl>
              <FormLabel>Discussion Type</FormLabel>
              <Select
                value={newAnnotation.type}
                onChange={(e) => setNewAnnotation(prev => ({ ...prev, type: e.target.value as Annotation['type'] }))}
                size="small"
              >
                <MenuItem value="comment">Comment</MenuItem>
                <MenuItem value="question">Question</MenuItem>
                <MenuItem value="suggestion">Suggestion</MenuItem>
                <MenuItem value="approval">Approval</MenuItem>
              </Select>
            </FormControl>

            {/* Mentions */}
            <Autocomplete
              multiple
              options={teamMembers}
              getOptionLabel={(option) => option.name}
              value={newAnnotation.mentions}
              onChange={(_, value) => setNewAnnotation(prev => ({ ...prev, mentions: value }))}
              renderInput={(params) => (
                <TextField {...params} label="Mention Team Members" placeholder="Type to search..." />
              )}
              renderTags={(value, getTagProps) =>
                value.map((option, index) => (
                  <Chip
                    label={`@${option.name}`}
                    {...getTagProps({ index })}
                    key={option.id}
                    size="small"
                    color="primary"
                  />
                ))
              }
            />

            {/* Tags */}
            <Autocomplete
              multiple
              freeSolo
              options={['critical', 'architecture', 'refactoring', 'pattern', 'requirements', 'performance', 'security']}
              value={newAnnotation.tags}
              onChange={(_, value) => setNewAnnotation(prev => ({ ...prev, tags: value }))}
              renderInput={(params) => (
                <TextField {...params} label="Tags" placeholder="Add tags..." />
              )}
              renderTags={(value, getTagProps) =>
                value.map((option, index) => (
                  <Chip
                    label={option}
                    {...getTagProps({ index })}
                    key={option}
                    size="small"
                    variant="outlined"
                  />
                ))
              }
            />
          </Box>
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setAnnotationDialogOpen(false)}>
            Cancel
          </Button>
          <Button
            onClick={handleAnnotationSubmit}
            variant="contained"
            startIcon={<SendOutlined />}
            disabled={!selectedFinding || !newAnnotation.content.trim()}
          >
            Post Discussion
          </Button>
        </DialogActions>
      </Dialog>

      {/* Create Review Dialog */}
      <Dialog
        open={reviewDialogOpen}
        onClose={() => setReviewDialogOpen(false)}
        maxWidth="md"
        fullWidth
      >
        <DialogTitle>Create Review</DialogTitle>
        <DialogContent>
          <Box display="flex" flexDirection="column" gap={2} pt={1}>
            <TextField
              fullWidth
              label="Review Title"
              value={newReview.title}
              onChange={(e) => setNewReview(prev => ({ ...prev, title: e.target.value }))}
              placeholder="Enter a descriptive title for this review..."
            />

            <TextField
              fullWidth
              multiline
              rows={3}
              label="Description"
              value={newReview.description}
              onChange={(e) => setNewReview(prev => ({ ...prev, description: e.target.value }))}
              placeholder="Describe what needs to be reviewed..."
            />

            <Grid container spacing={2}>
              <Grid item xs={12} sm={6}>
                <FormControl fullWidth>
                  <FormLabel>Assignee</FormLabel>
                  <Autocomplete
                    options={teamMembers}
                    getOptionLabel={(option) => `${option.name} (${option.role})`}
                    value={newReview.assignee}
                    onChange={(_, value) => setNewReview(prev => ({ ...prev, assignee: value }))}
                    renderInput={(params) => (
                      <TextField {...params} placeholder="Who should do this review?" />
                    )}
                  />
                </FormControl>
              </Grid>

              <Grid item xs={12} sm={6}>
                <FormControl fullWidth>
                  <FormLabel>Priority</FormLabel>
                  <Select
                    value={newReview.priority}
                    onChange={(e) => setNewReview(prev => ({ ...prev, priority: e.target.value as Review['priority'] }))}
                  >
                    <MenuItem value="low">Low</MenuItem>
                    <MenuItem value="medium">Medium</MenuItem>
                    <MenuItem value="high">High</MenuItem>
                    <MenuItem value="critical">Critical</MenuItem>
                  </Select>
                </FormControl>
              </Grid>
            </Grid>

            <TextField
              fullWidth
              type="date"
              label="Due Date"
              value={newReview.dueDate}
              onChange={(e) => setNewReview(prev => ({ ...prev, dueDate: e.target.value }))}
              InputLabelProps={{ shrink: true }}
            />

            <Autocomplete
              multiple
              options={findings}
              getOptionLabel={(option) => option.title}
              value={findings.filter(f => newReview.findings.includes(f.id))}
              onChange={(_, value) => setNewReview(prev => ({ ...prev, findings: value.map(v => v.id) }))}
              renderInput={(params) => (
                <TextField {...params} label="Related Findings" placeholder="Select findings to review..." />
              )}
            />
          </Box>
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setReviewDialogOpen(false)}>
            Cancel
          </Button>
          <Button
            onClick={handleReviewSubmit}
            variant="contained"
            startIcon={<AssignmentIndOutlined />}
            disabled={!newReview.title.trim() || !newReview.assignee}
          >
            Create Review
          </Button>
        </DialogActions>
      </Dialog>

      {/* Annotation context menu */}
      <Menu
        anchorEl={anchorEl}
        open={Boolean(anchorEl)}
        onClose={() => setAnchorEl(null)}
      >
        <MenuItem onClick={() => setAnchorEl(null)}>
          <ListItemIcon>
            <EditOutlined fontSize="small" />
          </ListItemIcon>
          Edit
        </MenuItem>
        <MenuItem onClick={() => setAnchorEl(null)}>
          <ListItemIcon>
            <PushPinOutlined fontSize="small" />
          </ListItemIcon>
          Pin
        </MenuItem>
        <MenuItem onClick={() => setAnchorEl(null)}>
          <ListItemIcon>
            <FlagOutlined fontSize="small" />
          </ListItemIcon>
          Report
        </MenuItem>
        <Divider />
        <MenuItem onClick={() => setAnchorEl(null)} sx={{ color: 'error.main' }}>
          <ListItemIcon>
            <DeleteOutlined fontSize="small" color="error" />
          </ListItemIcon>
          Delete
        </MenuItem>
      </Menu>
    </Box>
  );
};

export default CollaborationPanel;