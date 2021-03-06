import React from 'react';
import PropTypes from 'prop-types';
import styled from 'styled-components';
import { Link, useRoute } from 'wouter';
import * as Icons from './components/icons';

// TODO: use theme to pull colors...?
const NavBar = styled.nav`
  background-color: #0A0F12;
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  justify-content: center;
  list-style-type: none;
  width: 60px;
  height: 100%;

  padding: 0;
  margin: 0;

  grid-column: 1 / 1;
`;

const NavLink = styled.a`
  color: white;
  background-color: ${props => props.selected ? '#2C3033' : '#0A0F12'};
  padding: 16px;  
  text-align: right;
  text-decoration: none;
  font-size: 18px;
  width: 60px;
`;

const NavBarLink = (props) => {
  const params = useRoute(`${props.href}/:rest*`)[1];
  const isActive = params !== null;

  return (
    <Link {...props} href={props.href}>
      <NavLink selected={isActive}>{props.children}</NavLink>
    </Link>
  );
};

NavBarLink.propTypes = {
  children: PropTypes.node,
  href: PropTypes.string,
};

export const Nav = () => (
  <NavBar>
    <NavBarLink href='/app/activity'><Icons.Activity /></NavBarLink>
    <NavBarLink href='/app/schedules'><Icons.Box /></NavBarLink>
    <NavBarLink href='/app/settings'><Icons.Sliders /></NavBarLink>
  </NavBar>
);
