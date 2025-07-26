import { formatDistanceToNow } from 'date-fns';
import { CheckCircle, MessageSquare, Tag } from 'lucide-react';
import React from 'react';
import type { Question } from '../../types/support';

interface QuestionCardProps {
  question: Question;
  onClick: (question: Question) => void;
}

const QuestionCard: React.FC<QuestionCardProps> = ({ question, onClick }) => {
  const timeAgo = formatDistanceToNow(new Date(question.created_at), { addSuffix: true });

  return (
    <div
      onClick={() => onClick(question)}
      className="bg-secondary-900/50 rounded-lg shadow-sm border border-secondary-800 p-6 hover:shadow-md hover:bg-secondary-900/70 transition-all cursor-pointer"
    >
      <div className="flex justify-between items-start mb-3">
        <h3 className="text-lg font-semibold text-white line-clamp-2">
          {question.title}
        </h3>
        {question.is_answered && (
          <CheckCircle className="h-5 w-5 text-green-500 flex-shrink-0 ml-2" />
        )}
      </div>
      
      <p className="text-secondary-300 mb-4 line-clamp-3">
        {question.content}
      </p>
      
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
      
      <div className="flex justify-between items-center text-sm text-secondary-400">
        <div className="flex items-center space-x-4">
          <span>Asked by {question.author.username}</span>
          <span>{timeAgo}</span>
        </div>
        <div className="flex items-center space-x-2">
          <MessageSquare className="h-4 w-4" />
          <span>{question.answer_count} answers</span>
        </div>
      </div>
    </div>
  );
};

export default QuestionCard;
