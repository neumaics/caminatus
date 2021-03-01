import React from 'react';
import { Route, Switch } from 'wouter';

import { Dashboard } from './dashboard';
import { Schedules, Schedule, CreateSchedule } from './schedule';
import { Nav } from './nav';
import styled from 'styled-components';

const Container = styled.div`
  display: grid;
  grid-template-columns: 140px auto;
  grid-template-rows: 100%;
  height: 100vh;
  width: 100vw;
  overflow: hidden;
`;

const Content = styled.div`
  display: flex;
  margin: 1em;
  overflow-y: scroll;
`;

export const App = () => (<Container>
  <Nav />
  <Content>
    <Switch>
      <Route path='/' component={Dashboard} />
      <Route path='/app' component={Dashboard} />
      <Route path='/app/schedules' component={Schedules} />
      <Route path='/app/schedules/create' component={CreateSchedule} />
      <Route path='/app/schedules/:scheduleName' component={Schedule} />
    </Switch>
  </Content>
</Container>
);
