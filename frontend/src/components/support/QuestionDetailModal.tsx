import { formatDistanceToNow } from 'date-fns';
import { ArrowLeft, CheckCircle, Tag, ThumbsDown, ThumbsUp } from 'lucide-react';
import React, { useState } from 'react';
import { useUser } from '../../store/auth';
import type { Answer, AnswerCreate, Question } from '../../types/support';
import Button from '../ui/Button';

interface QuestionDetailModalProps {
  question: Question | null;
  answers: Answer[];
  onClose: () => void;
  onSubmitAnswer: (answer: AnswerCreate) => void;
  onVoteAnswer: (answerId: string, vote: 'up' | 'down') => void;
  onAcceptAnswer: (answerId: string) => void;
  isLoading?: boolean;
}

const QuestionDetailModal: React.FC<QuestionDetailModalProps> = ({
  question,
  answers,
  onClose,
  onSubmitAnswer,
  onVoteAnswer,
  onAcceptAnswer,
  // isLoading = false // unused parameter
}) => {
  const [answerContent, setAnswerContent] = useState('');
  const [isSubmitting, setIsSubmitting] = useState(false);
  const user = useUser();

  if (!question) return null;

  const handleSubmitAnswer = async (e: React.FormEvent) => {
    e.preventDefault();
    if (answerContent.trim()) {
      setIsSubmitting(true);
      await onSubmitAnswer({
        question_id: question.id,
        content: answerContent.trim()
      });
      setAnswerContent('');
      setIsSubmitting(false);
    }
  };

  const canAcceptAnswer = user?.user_id === question.author.user_id;

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50 p-4">
      <div className="bg-white dark:bg-gray-800 rounded-lg shadow-xl max-w-4xl w-full max-h-[90vh] overflow-y-auto">
        <div className="flex justify-between items-center p-6 border-b border-gray-200 dark:border-gray-700">
          <button
            onClick={onClose}
            className="flex items-center text-gray-600 hover:text-gray-900 dark:text-gray-400 dark:hover:text-white"
          >
            <ArrowLeft className="h-5 w-5 mr-2" />
            Back to Questions
          </button>
          {question.is_answered && (
            <div className="flex items-center text-green-600">
              <CheckCircle className="h-5 w-5 mr-2" />
              Answered
            </div>
          )}
        </div>
        
        <div className="p-6">
          {/* Question */}
          <div className="mb-8">
            <h1 className="text-2xl font-bold text-gray-900 dark:text-white mb-4">
              {question.title}
            </h1>
            
            {question.tags.length > 0 && (
              <div className="flex flex-wrap gap-2 mb-4">
                {question.tags.map(tag => (
                  <span
                    key={tag}
                    className="inline-flex items-center px-2 py-1 text-xs font-medium bg-primary-100 text-primary-800 rounded-full"
                  >
                    <Tag className="h-3 w-3 mr-1" />
                    {tag}
                  </span>
                ))}
              </div>
            )}
            
            <div className="prose dark:prose-invert max-w-none mb-4">
              <p className="whitespace-pre-wrap">{question.content}</p>
            </div>
            
            <div className="flex justify-between items-center text-sm text-gray-500 dark:text-gray-400">
              <span>Asked by {question.author.username}</span>
              <span>{formatDistanceToNow(new Date(question.created_at), { addSuffix: true })}</span>
            </div>
          </div>

          {/* Answers */}
          <div className="mb-8">
            <h2 className="text-xl font-semibold text-gray-900 dark:text-white mb-4">
              {answers.length} Answer{answers.length !== 1 ? 's' : ''}
            </h2>
            
            {answers.length === 0 ? (
              <p className="text-gray-500 dark:text-gray-400 text-center py-8">
                No answers yet. Be the first to help!
              </p>
            ) : (
              <div className="space-y-6">
                {answers.map(answer => (
                  <div
                    key={answer.id}
                    className={`border rounded-lg p-4 ${
                      answer.is_accepted
                        ? 'border-green-500 bg-green-50 dark:bg-green-900/20'
                        : 'border-gray-200 dark:border-gray-700'
                    }`}
                  >
                    {answer.is_accepted && (
                      <div className="flex items-center text-green-600 mb-2">
                        <CheckCircle className="h-4 w-4 mr-2" />
                        <span className="text-sm font-medium">Accepted Answer</span>
                      </div>
                    )}
                    
                    <div className="prose dark:prose-invert max-w-none mb-4">
                      <p className="whitespace-pre-wrap">{answer.content}</p>
                    </div>
                    
                    <div className="flex justify-between items-center">
                      <div className="flex items-center space-x-4">
                        <div className="flex items-center space-x-1">
                          <button
                            onClick={() => onVoteAnswer(answer.id, 'up')}
                            className="text-gray-500 hover:text-green-600 dark:text-gray-400 dark:hover:text-green-400"
                          >
                            <ThumbsUp className="h-4 w-4" />
                          </button>
                          <span className="text-sm font-medium">{answer.votes}</span>
                          <button
                            onClick={() => onVoteAnswer(answer.id, 'down')}
                            className="text-gray-500 hover:text-red-600 dark:text-gray-400 dark:hover:text-red-400"
                          >
                            <ThumbsDown className="h-4 w-4" />
                          </button>
                        </div>
                        
                        {canAcceptAnswer && !answer.is_accepted && !question.is_answered && (
                          <Button
                            onClick={() => onAcceptAnswer(answer.id)}
                            variant="outline"
                            size="sm"
                          >
                            Accept Answer
                          </Button>
                        )}
                      </div>
                      
                      <div className="text-sm text-gray-500 dark:text-gray-400">
                        <span>by {answer.author.username}</span>
                        <span className="ml-2">
                          {formatDistanceToNow(new Date(answer.created_at), { addSuffix: true })}
                        </span>
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            )}
          </div>

          {/* Answer Form */}
          {user?.is_verified && (
            <div className="border-t border-gray-200 dark:border-gray-700 pt-6">
              <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-4">
                Your Answer
              </h3>
              
              <form onSubmit={handleSubmitAnswer}>
                <textarea
                  value={answerContent}
                  onChange={(e) => setAnswerContent(e.target.value)}
                  rows={6}
                  className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-primary-500 focus:border-primary-500 dark:bg-gray-700 dark:text-white"
                  placeholder="Write your answer here..."
                  required
                />
                
                <div className="flex justify-end mt-4">
                  <Button
                    type="submit"
                    disabled={isSubmitting || !answerContent.trim()}
                  >
                    {isSubmitting ? 'Posting...' : 'Post Answer'}
                  </Button>
                </div>
              </form>
            </div>
          )}
        </div>
      </div>
    </div>
  );
};

export default QuestionDetailModal;
