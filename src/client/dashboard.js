import React, { useEffect, useState } from 'react';
import * as Scale from '@visx/scale';
import styled from 'styled-components';
import {
  AnimatedAxis,
  AnimatedGrid,
  AnimatedLineSeries,
  XYChart,
  Tooltip,
} from '@visx/xychart';
import { ParentSize } from '@visx/responsive';

import { byName, toGraphN } from './schedule/common';

const GraphContainer = styled.div`
  height: 100%;
  width: 100%;
  background: #0A0F12;
  border-radius: 5px;

  .axis-label {
    fill: white;
    font-family: "Work Sans", sans-serif;
  }

  .axis-tick {
    > * g {
      fill: white;
      font-family: "Work Sans", sans-serif;

    }
  }
`;

const accessors = {
  xAccessor: d => d.x,
  yAccessor: d => d.y,
};

export const Dashboard = () => {
  const [schedule, setSchedule] = useState({ name: '' });
  const [liveTemp, setLiveTemp] = useState([]);
  const [graphData, setGraphData] = useState([]);

  useEffect(() => {
    byName('fast', true).then((s) => {
      setGraphData(toGraphN(s));
      setSchedule(s);
    });
  }, []);

  return (
    <GraphContainer>
      {/* <span>{schedule.name}</span> */}
      <ParentSize>
        {parent => <XYChart
          height={parent.height}
          width={parent.width}
          xScale={{ type: 'linear' }}
          yScale={{ type: 'linear' }}>
          <AnimatedAxis
            orientation='bottom'
            label='time (seconds)'
            labelClassName='axis-label'
            tickClassName='axis-tick'
            numTicks={4}
          />
          <AnimatedAxis
            scale={Scale.scaleLinear({
              domain: [0, 2000],
              range: [0, parent.height],
              nice: true
            })}
            label='Temperature (°C)'
            orientation='left'
            labelClassName='axis-label'
            tickClassName='axis-tick'
          />
          <AnimatedGrid columns={false} numTicks={4} />
          <AnimatedLineSeries dataKey='set' data={graphData} {...accessors} />
          <AnimatedLineSeries dataKey='Line 2' data={liveTemp} {...accessors} />
          <Tooltip
            snapTooltipToDatumX
            snapTooltipToDatumY
            showVerticalCrosshair
            showSeriesGlyphs
            renderTooltip={({ tooltipData, colorScale }) => (
              <div>
                <div style={{ color: colorScale(tooltipData.nearestDatum.key) }}>
                  {tooltipData.nearestDatum.key}
                </div>
                {accessors.xAccessor(tooltipData.nearestDatum.datum)}
                {', '}
                {accessors.yAccessor(tooltipData.nearestDatum.datum)}
              </div>
            )}
          />

        </XYChart>}
      </ParentSize>
    </GraphContainer>
  );
};
