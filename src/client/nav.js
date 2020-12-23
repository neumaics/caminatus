import React from 'react';
import styled from 'styled-components';
import { Link, useRoute } from 'wouter';

// TODO: use theme to pull colors...?
const NavBar = styled.nav`
  background-color: #0A0F12;
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  justify-content: center;
  list-style-type: none;
  width: 140px;
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
  width: 140px;
`;

const NavBarLink = (props) => {
  const [_match, params] = useRoute(`${props.href}/:rest*`);
  const isActive = params !== null;

  return (
    <Link {...props} href={props.href}>
      <NavLink selected={isActive}>{props.children}</NavLink>
    </Link>
  );
};

export const Nav = () => (
  <NavBar>
    <NavBarLink href='/app/schedules'>Schedules</NavBarLink>
  </NavBar>);