// All svg from https://feathericons.com
import React from 'react';
import PropTypes from 'prop-types';

export const Activity = ({ className }) => {
  return (
    <svg
      xmlns='http://www.w3.org/2000/svg'
      width='24'
      height='24'
      viewBox='0 0 24 24'
      fill='none'
      stroke='currentColor'
      strokeWidth='2'
      strokeLinecap='round'
      strokeLinejoin='round'
      className={className}>
      <polyline points='22 12 18 12 15 21 9 3 6 12 2 12'></polyline>
    </svg>
  );
};
   
Activity.propTypes = {
  className: PropTypes.string,
};

export const Sliders = ({ className }) => (
  <svg
    xmlns='http://www.w3.org/2000/svg'
    width='24'
    height='24'
    viewBox='0 0 24 24'
    fill='none'
    stroke='currentColor'
    strokeWidth='2'
    strokeLinecap='round'
    strokeLinejoin='round'
    className={className}>
    <line x1='4' y1='21' x2='4' y2='14'></line>
    <line x1='4' y1='10' x2='4' y2='3'></line>
    <line x1='12' y1='21' x2='12' y2='12'></line>
    <line x1='12' y1='8' x2='12' y2='3'></line>
    <line x1='20' y1='21' x2='20' y2='16'></line>
    <line x1='20' y1='12' x2='20' y2='3'></line>
    <line x1='1' y1='14' x2='7' y2='14'></line>
    <line x1='9' y1='8' x2='15' y2='8'></line>
    <line x1='17' y1='16' x2='23' y2='16'></line>
  </svg>
);

Sliders.propTypes = {
  className: PropTypes.string,
};

export const Box = ({ className }) => (
  <svg xmlns='http://www.w3.org/2000/svg'
    width='24'
    height='24'
    viewBox='0 0 24 24'
    fill='none'
    stroke='currentColor'
    strokeWidth='2'
    strokeLinecap='round'
    strokeLinejoin='round'
    className={className}>
    <path d='M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z'></path>
    <polyline points='3.27 6.96 12 12.01 20.73 6.96'></polyline>
    <line x1='12' y1='22.08' x2='12' y2='12'></line>
  </svg>
);

Box.propTypes = {
  className: PropTypes.string,
};

export const ArrowLeft = ({ className }) => (
  <svg
    xmlns='http://www.w3.org/2000/svg'
    width='24'
    height='24'
    viewBox='0 0 24 24'
    fill='none'
    stroke='currentColor'
    strokeWidth='2'
    strokeLinecap='round'
    strokeLinejoin='round'
    className={className}>
    <line x1='19' y1='12' x2='5' y2='12'></line>
    <polyline points='12 19 5 12 12 5'></polyline>
  </svg>
);

ArrowLeft.propTypes = {
  className: PropTypes.string,
};

export const FilePlus = ({ className }) => (
  <svg
    xmlns='http://www.w3.org/2000/svg'
    width='24'
    height='24'
    viewBox='0 0 24 24'
    fill='none'
    stroke='currentColor'
    strokeWidth='2'
    strokeLinecap='round'
    strokeLinejoin='round'
    className={className}>
    <path d='M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z'></path>
    <polyline points='14 2 14 8 20 8'></polyline>
    <line x1='12' y1='18' x2='12' y2='12'></line>
    <line x1='9' y1='15' x2='15' y2='15'></line>
  </svg>
);

FilePlus.propTypes = {
  className: PropTypes.string,
};
