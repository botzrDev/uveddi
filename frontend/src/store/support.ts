import { create } from 'zustand';
import type { Answer, AnswerCreate, Question, QuestionCreate } from '../types/support';

interface SupportState {
  questions: Question[];
  answers: Record<string, Answer[]>; // questionId -> answers
  isLoading: boolean;
  error: string | null;
}

interface SupportActions {
  // Questions
  setQuestions: (questions: Question[]) => void;
  addQuestion: (question: Question) => void;
  updateQuestion: (questionId: string, updates: Partial<Question>) => void;
  
  // Answers
  setAnswers: (questionId: string, answers: Answer[]) => void;
  addAnswer: (answer: Answer) => void;
  updateAnswer: (answerId: string, updates: Partial<Answer>) => void;
  
  // State management
  setLoading: (loading: boolean) => void;
  setError: (error: string | null) => void;
  clearError: () => void;
}

type SupportStore = SupportState & SupportActions;

export const useSupportStore = create<SupportStore>()((set) => ({
  // Initial state
  questions: [],
  answers: {},
  isLoading: false,
  error: null,

  // Actions
  setQuestions: (questions) => set({ questions, error: null }),
  
  addQuestion: (question) => set((state) => ({
    questions: [question, ...state.questions],
    error: null
  })),
  
  updateQuestion: (questionId, updates) => set((state) => ({
    questions: state.questions.map(q => 
      q.id === questionId ? { ...q, ...updates } : q
    ),
    error: null
  })),
  
  setAnswers: (questionId, answers) => set((state) => ({
    answers: { ...state.answers, [questionId]: answers },
    error: null
  })),
  
  addAnswer: (answer) => set((state) => ({
    answers: {
      ...state.answers,
      [answer.question_id]: [
        ...(state.answers[answer.question_id] || []),
        answer
      ]
    },
    error: null
  })),
  
  updateAnswer: (answerId, updates) => set((state) => {
    const newAnswers = { ...state.answers };
    Object.keys(newAnswers).forEach(questionId => {
      newAnswers[questionId] = newAnswers[questionId].map(a =>
        a.id === answerId ? { ...a, ...updates } : a
      );
    });
    return { answers: newAnswers, error: null };
  }),
  
  setLoading: (loading) => set({ isLoading: loading }),
  setError: (error) => set({ error }),
  clearError: () => set({ error: null })
}));

// Mock API functions - replace with real API calls
export const supportApi = {
  // Questions
  async getQuestions(): Promise<Question[]> {
    // Mock data - replace with real API call
    return [
      {
        id: '1',
        title: 'How do I set up my first project?',
        content: 'I\'m new to Uveddi and would like to know how to set up my first project. What are the basic steps?',
        author: { username: 'newuser123', user_id: 1 },
        created_at: new Date(Date.now() - 2 * 24 * 60 * 60 * 1000).toISOString(),
        updated_at: new Date(Date.now() - 2 * 24 * 60 * 60 * 1000).toISOString(),
        answer_count: 2,
        is_answered: true,
        tags: ['setup', 'beginner']
      },
      {
        id: '2',
        title: 'Architecture analysis not detecting issues',
        content: 'My architecture analysis is running but not detecting any issues in my codebase. I know there are some problems. What could be wrong?',
        author: { username: 'developer456', user_id: 2 },
        created_at: new Date(Date.now() - 1 * 24 * 60 * 60 * 1000).toISOString(),
        updated_at: new Date(Date.now() - 1 * 24 * 60 * 60 * 1000).toISOString(),
        answer_count: 1,
        is_answered: false,
        tags: ['analysis', 'troubleshooting']
      }
    ];
  },

  async createQuestion(question: QuestionCreate): Promise<Question> {
    // Mock response - replace with real API call
    return {
      id: Math.random().toString(36).substr(2, 9),
      title: question.title,
      content: question.content,
      author: { username: 'current_user', user_id: 999 }, // Should come from auth
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
      answer_count: 0,
      is_answered: false,
      tags: question.tags
    };
  },

  // Answers
  async getAnswers(questionId: string): Promise<Answer[]> {
    // Mock data - replace with real API call
    if (questionId === '1') {
      return [
        {
          id: 'a1',
          question_id: '1',
          content: 'Welcome to Uveddi! To set up your first project, go to the Dashboard and click "New Project". Then connect your repository URL and configure your analysis settings.',
          author: { username: 'supportteam', user_id: 100 },
          created_at: new Date(Date.now() - 1 * 24 * 60 * 60 * 1000).toISOString(),
          updated_at: new Date(Date.now() - 1 * 24 * 60 * 60 * 1000).toISOString(),
          is_accepted: true,
          votes: 5
        },
        {
          id: 'a2',
          question_id: '1',
          content: 'Also check out our Getting Started guide in the documentation. It has step-by-step instructions with screenshots.',
          author: { username: 'helper789', user_id: 101 },
          created_at: new Date(Date.now() - 20 * 60 * 60 * 1000).toISOString(),
          updated_at: new Date(Date.now() - 20 * 60 * 60 * 1000).toISOString(),
          is_accepted: false,
          votes: 2
        }
      ];
    } else if (questionId === '2') {
      return [
        {
          id: 'a3',
          question_id: '2',
          content: 'Check your project configuration. Make sure the file patterns are correct and that your repository has the supported language files.',
          author: { username: 'techexpert', user_id: 102 },
          created_at: new Date(Date.now() - 12 * 60 * 60 * 1000).toISOString(),
          updated_at: new Date(Date.now() - 12 * 60 * 60 * 1000).toISOString(),
          is_accepted: false,
          votes: 1
        }
      ];
    }
    return [];
  },

  async createAnswer(answer: AnswerCreate): Promise<Answer> {
    // Mock response - replace with real API call
    return {
      id: Math.random().toString(36).substr(2, 9),
      question_id: answer.question_id,
      content: answer.content,
      author: { username: 'current_user', user_id: 999 }, // Should come from auth
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
      is_accepted: false,
      votes: 0
    };
  },

  async voteAnswer(answerId: string, vote: 'up' | 'down'): Promise<void> {
    // Mock implementation - replace with real API call
    console.log(`Voting ${vote} on answer ${answerId}`);
  },

  async acceptAnswer(answerId: string): Promise<void> {
    // Mock implementation - replace with real API call
    console.log(`Accepting answer ${answerId}`);
  }
};
