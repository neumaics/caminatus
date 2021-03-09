import React from 'react';
import PropTypes from 'prop-types';
import { Link } from 'wouter';

import styled from 'styled-components';

const colors = {
  default: '#3498DB',
  error: '#E74C3C',
  warn: '#F1C40F',
  success: '#2ECC71',
};

const Button = () => <button></button>;

export const StyledButton = styled(Button)`
`;

export const LinkButtonDefault = ({ className, children, href }) => (
  <Link href={href}>
    <button className={className}>
      {children}
    </button>
  </Link>
);

LinkButtonDefault.propTypes = {
  className: PropTypes.string,
  children: PropTypes.node,
  context: PropTypes.oneOf(Object.keys(colors)),
  href: PropTypes.string,
  inverted: PropTypes.bool,
};

const FormButtonDefault = (props) => {
  return (<button
    type={props.type}
    onClick={props.onClick}
    className={props.className}
  >
    {props.children}
  </button>);
};

FormButtonDefault.propTypes = {
  className: PropTypes.string,
  children: PropTypes.node,
  context: PropTypes.oneOf(Object.keys(colors)),
  inverted: PropTypes.bool,
  onClick: PropTypes.func,
  type: PropTypes.oneOf(['button', 'submit']),
};

FormButtonDefault.defaultProps = {
  context: 'default',
  inverted: false,
  type: 'button',
};

export const FormButton = styled(FormButtonDefault)`
  box-sizing: border-box;
  font-family: inherit;
  padding: 0;
  cursor: pointer;
  display: inline-flex;
  align-self: start;
  align-items: center;
  justify-content: center;

  background-color: ${({ context, inverted }) => inverted ? 'transparent' : colors[context]};
  color: ${({ context, inverted }) => inverted ? colors[context] : '#FFF'};

  border-color: ${({ context, inverted }) => inverted ? colors[context] : 'transparent'};
  border-radius: 2px;
  border-width: 2px;
  border-style: solid;

  padding: 0.25em 0.75em;
  min-width: 0;
  min-height: 32px;
  margin: 0 4px;

  text-align: center;
  line-height: 1.1;
  font-size: 16px;
`;

export const LinkButton = styled(LinkButtonDefault)`
  box-sizing: border-box;
  font-family: inherit;
  padding: 0;
  cursor: pointer;
  display: inline-flex;
  align-self: start;
  align-items: center;
  justify-content: center;

  background-color: ${({ context, inverted }) => inverted ? 'transparent' : colors[context]};
  color: ${({ context, inverted }) => inverted ? colors[context] : '#FFF'};

  border-color: ${({ context, inverted }) => inverted ? colors[context] : 'transparent'};
  border-radius: 2px;
  border-width: 2px;
  border-style: solid;

  padding: 0.25em 0.75em;
  min-width: 0;
  min-height: 32px;
  margin: 0 4px;

  text-align: center;
  line-height: 1.1;
  font-size: 16px;

  > a {
    color: ${({ context, inverted }) => inverted ? colors[context] : '#FFF'};
  }
`;
