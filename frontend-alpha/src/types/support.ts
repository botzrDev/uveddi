// Types for Q&A system
export interface Question {
  id: string;
  title: string;
  content: string;
  author: {
    username: string;
    user_id: number;
  };
  created_at: string;
  updated_at: string;
  answer_count: number;
  is_answered: boolean;
  tags: string[];
}

export interface Answer {
  id: string;
  question_id: string;
  content: string;
  author: {
    username: string;
    user_id: number;
  };
  created_at: string;
  updated_at: string;
  is_accepted: boolean;
  votes: number;
}

export interface QuestionCreate {
  title: string;
  content: string;
  tags: string[];
}

export interface AnswerCreate {
  question_id: string;
  content: string;
}
