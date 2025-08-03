import { Filter, Plus, Search } from 'lucide-react';
import React, { useEffect, useState } from 'react';
import { Link } from 'react-router-dom';
import Footer from '../components/layout/Footer';
import Header from '../components/layout/Header';
import AskQuestionModal from '../components/support/AskQuestionModal';
import QuestionCard from '../components/support/QuestionCard';
import QuestionDetailModal from '../components/support/QuestionDetailModal';
import Button from '../components/ui/Button';
import { useIsAuthenticated, useUser } from '../store/auth';
import { supportApi, useSupportStore } from '../store/support';
import type { AnswerCreate, Question, QuestionCreate } from '../types/support';

const importantDocs = [
  { name: 'Getting Started', url: '/docs/getting-started' },
  { name: 'FAQ', url: '/docs/faq' },
  { name: 'Deployment Guide', url: '/docs/deployment-guide' },
  { name: 'Community Guidelines', url: '/docs/community-guidelines' },
];

const CommunitySupportPage: React.FC = () => {
  const isAuthenticated = useIsAuthenticated();
  const user = useUser();
  const isVerified = user?.is_verified;
  
  const {
    questions,
    answers,
    isLoading,
    error,
    setQuestions,
    addQuestion,
    updateQuestion,
    setAnswers,
    addAnswer,
    updateAnswer,
    setLoading,
    setError,
    clearError
  } = useSupportStore();

  const [searchTerm, setSearchTerm] = useState('');
  const [filterTag, setFilterTag] = useState('');
  const [showAskModal, setShowAskModal] = useState(false);
  const [selectedQuestion, setSelectedQuestion] = useState<Question | null>(null);

  // Load questions on component mount
  useEffect(() => {
    loadQuestions();
  }, []);

  // Scroll to top when component mounts (instant scroll to avoid jarring animation)
  useEffect(() => {
    window.scrollTo(0, 0);
  }, []);

  // Load answers when a question is selected
  useEffect(() => {
    if (selectedQuestion && !answers[selectedQuestion.id]) {
      loadAnswers(selectedQuestion.id);
    }
  }, [selectedQuestion, answers]);

  const loadQuestions = async () => {
    try {
      setLoading(true);
      const questionsData = await supportApi.getQuestions();
      setQuestions(questionsData);
    } catch (err) {
      setError('Failed to load questions');
    } finally {
      setLoading(false);
    }
  };

  const loadAnswers = async (questionId: string) => {
    try {
      const answersData = await supportApi.getAnswers(questionId);
      setAnswers(questionId, answersData);
    } catch (err) {
      setError('Failed to load answers');
    }
  };

  const handleAskQuestion = async (questionData: QuestionCreate) => {
    try {
      const newQuestion = await supportApi.createQuestion(questionData);
      addQuestion(newQuestion);
      setShowAskModal(false);
    } catch (err) {
      setError('Failed to create question');
    }
  };

  const handleAnswerQuestion = async (answerData: AnswerCreate) => {
    try {
      const newAnswer = await supportApi.createAnswer(answerData);
      addAnswer(newAnswer);
      // Update question answer count
      if (selectedQuestion) {
        updateQuestion(selectedQuestion.id, {
          answer_count: selectedQuestion.answer_count + 1
        });
      }
    } catch (err) {
      setError('Failed to post answer');
    }
  };

  const handleVoteAnswer = async (answerId: string, vote: 'up' | 'down') => {
    try {
      await supportApi.voteAnswer(answerId, vote);
      // Update answer vote count (simplified logic)
      updateAnswer(answerId, {
        votes: vote === 'up' ? 1 : -1 // This should be handled properly on the backend
      });
    } catch (err) {
      setError('Failed to vote on answer');
    }
  };

  const handleAcceptAnswer = async (answerId: string) => {
    try {
      await supportApi.acceptAnswer(answerId);
      updateAnswer(answerId, { is_accepted: true });
      if (selectedQuestion) {
        updateQuestion(selectedQuestion.id, { is_answered: true });
      }
    } catch (err) {
      setError('Failed to accept answer');
    }
  };

  const filteredQuestions = questions.filter(question => {
    const matchesSearch = question.title.toLowerCase().includes(searchTerm.toLowerCase()) ||
                         question.content.toLowerCase().includes(searchTerm.toLowerCase());
    const matchesTag = !filterTag || question.tags.includes(filterTag);
    return matchesSearch && matchesTag;
  });

  const allTags = [...new Set(questions.flatMap(q => q.tags))];

  return (
    <div className="min-h-screen bg-secondary-950">
      <Header variant="landing" />
      <main className="container mx-auto px-6 py-12">
        <div className="mb-8">
          <h1 className="text-3xl font-bold mb-4 text-white">Community Support</h1>
          <p className="text-secondary-300">
            Welcome to the Uveddi Community Support page! Find helpful resources and get answers from the community.
          </p>
        </div>        {/* Error Display */}
        {error && (
          <div className="mb-6 p-4 bg-red-900/20 border border-red-500 text-red-300 rounded">
            {error}
            <button
              onClick={clearError}
              className="ml-2 text-red-400 hover:text-red-200"
            >
              ×
            </button>
          </div>
        )}

        {/* Helpful Documents */}
        <div className="mb-8 bg-secondary-900/50 rounded-lg p-6">
          <h2 className="text-xl font-semibold mb-4 text-white">Helpful Documents</h2>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            {importantDocs.map(doc => (
              <a
                key={doc.url}
                href={doc.url}
                className="text-primary-400 hover:text-primary-300 transition-colors"
              >
                {doc.name}
              </a>
            ))}
          </div>
        </div>        {/* Community Q&A */}
        <div className="mb-8">
          <div className="flex justify-between items-center mb-6">
            <h2 className="text-2xl font-semibold text-white">Community Q&A</h2>
            {isAuthenticated && isVerified && (
              <Button
                onClick={() => setShowAskModal(true)}
                className="flex items-center"
              >
                <Plus className="h-4 w-4 mr-2" />
                Ask Question
              </Button>
            )}
          </div>

          {isAuthenticated && isVerified ? (
            <>
              {/* Search and Filter */}
              <div className="mb-6 flex flex-col sm:flex-row gap-4">
                <div className="flex-1 relative">
                  <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 text-gray-400 h-4 w-4" />
                  <input
                    type="text"
                    value={searchTerm}
                    onChange={(e) => setSearchTerm(e.target.value)}
                    placeholder="Search questions..."
                    className="w-full pl-10 pr-4 py-2 border border-secondary-700 bg-secondary-800 text-white rounded-md focus:outline-none focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
                  />
                </div>
                <div className="flex items-center space-x-2">
                  <Filter className="h-4 w-4 text-gray-400" />
                  <select
                    value={filterTag}
                    onChange={(e) => setFilterTag(e.target.value)}
                    className="px-3 py-2 border border-secondary-700 bg-secondary-800 text-white rounded-md focus:outline-none focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
                  >
                    <option value="">All Tags</option>
                    {allTags.map(tag => (
                      <option key={tag} value={tag}>{tag}</option>
                    ))}
                  </select>
                </div>
              </div>

              {/* Questions List */}
              {isLoading ? (
                <div className="text-center py-8">
                  <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary-500 mx-auto"></div>
                  <p className="mt-2 text-secondary-300">Loading questions...</p>
                </div>
              ) : filteredQuestions.length === 0 ? (
                <div className="text-center py-8 text-secondary-300">
                  {searchTerm || filterTag ? 'No questions match your search criteria.' : 'No questions yet. Be the first to ask!'}
                </div>
              ) : (
                <div className="space-y-4">
                  {filteredQuestions.map(question => (
                    <QuestionCard
                      key={question.id}
                      question={question}
                      onClick={setSelectedQuestion}
                    />
                  ))}
                </div>
              )}
            </>
          ) : (
            <div className="text-center py-8 bg-secondary-900/50 rounded-lg">
              <p className="text-secondary-300 mb-4">
                You must be signed in with a verified account to view and post questions.
              </p>
              <div className="space-x-4">
                <Link
                  to="/login"
                  className="text-primary-400 hover:text-primary-300 transition-colors"
                >
                  Sign In
                </Link>
                <Link
                  to="/register"
                  className="text-primary-400 hover:text-primary-300 transition-colors"
                >
                  Register
                </Link>
              </div>
            </div>
          )}
        </div>

      {/* Modals */}
      <AskQuestionModal
        isOpen={showAskModal}
        onClose={() => setShowAskModal(false)}
        onSubmit={handleAskQuestion}
      />

      <QuestionDetailModal
        question={selectedQuestion}
        answers={selectedQuestion ? answers[selectedQuestion.id] || [] : []}
        onClose={() => setSelectedQuestion(null)}
        onSubmitAnswer={handleAnswerQuestion}
        onVoteAnswer={handleVoteAnswer}
        onAcceptAnswer={handleAcceptAnswer}
      />
      </main>
      <Footer />
    </div>
  );
};

export default CommunitySupportPage;
