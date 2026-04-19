import { useState } from 'react';
import { C } from '../theme';
import { Divider } from '../components';

export function LayoutPage() {
  const [showHidden, setShowHidden] = useState(false);
  const [gap, setGap] = useState(8);
  const [padding, setPadding] = useState(12);
  const [noRounding, setNoRounding] = useState(false);
  const [rounded, setRounded] = useState(true);
  const [circle, setCircle] = useState(false);

  return (
    <view display="flex" flexDir="col" gap={0} h="full" scrollable>
      <view
        display="flex"
        flexDir="col"
        px={24}
        py={16}
        borderBottom={1}
        borderColor={C.border}
      >
        <text fontSize={20} fontWeight={800} color={C.text}>
          Layout Lab
        </text>
        <text fontSize={12} color={C.textMuted}>
          Flex, nesting, borders, rounding, opacity, visibility
        </text>
      </view>

      <view display="flex" flexDir="col" gap={24} p={24}>
        <view display="flex" flexDir="col" gap={12}>
          <text fontSize={14} fontWeight={700} color={C.text}>
            Flexbox — justify variants
          </text>
          {(['center', 'between', 'around', 'evenly'] as const).map((j) => (
            <view key={j} display="flex" flexDir="col" gap={4}>
              <text fontSize={11} fontWeight={600} color={C.textMuted}>
                justify="{j}"
              </text>
              <view
                display="flex"
                flexDir="row"
                justify={j}
                bg={C.surface2}
                rounded={8}
                p={12}
                border={1}
                borderColor={C.border}
              >
                {[C.accentHi, C.primaryHi, C.successHi, C.warningHi].map(
                  (c, i) => (
                    <view key={i} w={36} h={36} bg={c} rounded={4} />
                  ),
                )}
              </view>
            </view>
          ))}
        </view>

        <Divider />

        <view display="flex" flexDir="col" gap={12}>
          <text fontSize={14} fontWeight={700} color={C.text}>
            Flexbox — items variants
          </text>
          <view display="flex" flexDir="row" gap={12}>
            {(['start', 'center', 'end', 'stretch'] as const).map((a) => (
              <view key={a} display="flex" flexDir="col" gap={4} flex={1}>
                <text fontSize={11} fontWeight={600} color={C.textMuted}>
                  items="{a}"
                </text>
                <view
                  display="flex"
                  flexDir="row"
                  items={a}
                  bg={C.surface2}
                  rounded={8}
                  p={10}
                  h={70}
                  border={1}
                  borderColor={C.border}
                  gap={4}
                >
                  {a === 'stretch' ? (
                    <>
                      <view w={24} bg={C.accentHi} rounded={4} />
                      <view w={24} bg={C.primaryHi} rounded={4} />
                      <view w={24} bg={C.successHi} rounded={4} />
                    </>
                  ) : (
                    <>
                      <view w={24} h={24} bg={C.accentHi} rounded={4} />
                      <view w={24} h={36} bg={C.primaryHi} rounded={4} />
                      <view w={24} h={16} bg={C.successHi} rounded={4} />
                    </>
                  )}
                </view>
              </view>
            ))}
          </view>
        </view>

        <Divider />

        <view display="flex" flexDir="col" gap={12}>
          <text fontSize={14} fontWeight={700} color={C.text}>
            Per-corner border-radius
          </text>
          <view display="flex" flexDir="row" gap={12} items="center">
            <view display="flex" flexDir="col" items="center" gap={4}>
              <view w={60} h={60} bg={C.accent} roundedTL={24} />
              <text fontSize={10} color={C.textMuted}>
                TL
              </text>
            </view>
            <view display="flex" flexDir="col" items="center" gap={4}>
              <view w={60} h={60} bg={C.primary} roundedTR={24} />
              <text fontSize={10} color={C.textMuted}>
                TR
              </text>
            </view>
            <view display="flex" flexDir="col" items="center" gap={4}>
              <view w={60} h={60} bg={C.success} roundedBR={24} />
              <text fontSize={10} color={C.textMuted}>
                BR
              </text>
            </view>
            <view display="flex" flexDir="col" items="center" gap={4}>
              <view w={60} h={60} bg={C.warning} roundedBL={24} />
              <text fontSize={10} color={C.textMuted}>
                BL
              </text>
            </view>
            <view display="flex" flexDir="col" items="center" gap={4}>
              <view w={60} h={60} bg={C.accent} roundedTL={24} roundedBR={24} />
              <text fontSize={10} color={C.textMuted}>
                TL+BR
              </text>
            </view>
            <view display="flex" flexDir="col" items="center" gap={4}>
              <view w={60} h={60} bg={C.danger} roundedTR={24} roundedBL={24} />
              <text fontSize={10} color={C.textMuted}>
                TR+BL
              </text>
            </view>
          </view>
        </view>

        <Divider />

        <view display="flex" flexDir="col" gap={12}>
          <text fontSize={14} fontWeight={700} color={C.text}>
            Per-side borders
          </text>
          <view display="flex" flexDir="row" gap={12} items="center">
            {[
              { side: 'Top', prop: { borderTop: 3 }, color: C.accentHi },
              { side: 'Right', prop: { borderRight: 3 }, color: C.primaryHi },
              { side: 'Bottom', prop: { borderBottom: 3 }, color: C.successHi },
              { side: 'Left', prop: { borderLeft: 3 }, color: C.warningHi },
              { side: 'All', prop: { border: 2 }, color: C.accentHi },
            ].map(({ side, prop, color }) => (
              <view
                key={side}
                display="flex"
                flexDir="col"
                items="center"
                gap={4}
              >
                <view
                  w={60}
                  h={60}
                  bg={C.surface2}
                  rounded={8}
                  borderColor={color}
                  {...prop}
                />
                <text fontSize={10} color={C.textMuted}>
                  {side}
                </text>
              </view>
            ))}
          </view>
        </view>

        <Divider />

        <view display="flex" flexDir="col" gap={12}>
          <text fontSize={14} fontWeight={700} color={C.text}>
            Opacity scale
          </text>
          <view display="flex" flexDir="row" gap={8} items="center">
            {[1, 0.8, 0.6, 0.4, 0.2, 0.1].map((op) => (
              <view
                key={op}
                display="flex"
                flexDir="col"
                items="center"
                gap={4}
              >
                <view
                  w={52}
                  h={52}
                  bg={C.accent}
                  rounded={8}
                  opacity={op}
                  display="flex"
                  items="center"
                  justify="center"
                >
                  <text fontSize={11} fontWeight={700} color="#fff">
                    {op}
                  </text>
                </view>
              </view>
            ))}
          </view>
        </view>

        <Divider />

        <view display="flex" flexDir="col" gap={12}>
          <view display="flex" flexDir="row" items="center" gap={20}>
            <text fontSize={14} fontWeight={700} color={C.text}>
              Dynamic gap / padding
            </text>
            <view display="flex" flexDir="row" items="center" gap={8}>
              <button
                onClick={() => setGap((g) => Math.max(2, g - 2))}
                px={10}
                py={4}
                bg={C.surface3}
                hover:bg={C.surface4}
                rounded={4}
                border={1}
                borderColor={C.border}
                cursor="pointer"
              >
                <text fontSize={13} color={C.text}>
                  gap−
                </text>
              </button>
              <text fontSize={12} color={C.accentHi}>
                gap={gap}
              </text>
              <button
                onClick={() => setGap((g) => Math.min(40, g + 2))}
                px={10}
                py={4}
                bg={C.surface3}
                hover:bg={C.surface4}
                rounded={4}
                border={1}
                borderColor={C.border}
                cursor="pointer"
              >
                <text fontSize={13} color={C.text}>
                  gap+
                </text>
              </button>
              <button
                onClick={() => setPadding((p) => Math.max(4, p - 4))}
                px={10}
                py={4}
                bg={C.surface3}
                hover:bg={C.surface4}
                rounded={4}
                border={1}
                borderColor={C.border}
                cursor="pointer"
              >
                <text fontSize={13} color={C.text}>
                  p−
                </text>
              </button>
              <text fontSize={12} color={C.primaryHi}>
                p={padding}
              </text>
              <button
                onClick={() => setPadding((p) => Math.min(40, p + 4))}
                px={10}
                py={4}
                bg={C.surface3}
                hover:bg={C.surface4}
                rounded={4}
                border={1}
                borderColor={C.border}
                cursor="pointer"
              >
                <text fontSize={13} color={C.text}>
                  p+
                </text>
              </button>
            </view>
          </view>
          <view
            display="flex"
            flexDir="row"
            gap={gap}
            p={padding}
            bg={C.surface2}
            rounded={8}
            border={1}
            borderColor={C.border}
          >
            {['A', 'B', 'C', 'D', 'E'].map((l, i) => (
              <view
                key={l}
                flex={1}
                p={padding}
                bg={
                  [
                    C.accentDim,
                    C.primaryDim,
                    C.successDim,
                    '#422006',
                    C.dangerDim,
                  ][i]
                }
                rounded={8}
                display="flex"
                items="center"
                justify="center"
              >
                <text fontSize={16} fontWeight={800} color={C.text}>
                  {l}
                </text>
              </view>
            ))}
          </view>
        </view>

        <Divider />

        <view display="flex" flexDir="col" gap={12}>
          <view display="flex" flexDir="col" gap={4}>
            <text fontSize={14} fontWeight={700} color={C.text}>
              Buttons
            </text>
            <text fontSize={12} color={C.textMuted}>
              Various button configurations and property combinations
            </text>
          </view>

          <view display="flex" flexDir="row" gap={12}>
            <view
              display="flex"
              flexDir="col"
              gap={8}
              flex={1}
              minW={200}
              p={16}
              bg={C.surface2}
              rounded={12}
              border={1}
              borderColor={C.border}
            >
              <view display="flex" flexDir="col" gap={4}>
                <text fontSize={13} fontWeight={600} color={C.accentHi}>
                  Default
                </text>
                <text fontSize={12} color={C.textMuted}>
                  No properties set
                </text>
              </view>
              <view
                display="flex"
                items="center"
                justify="center"
                p={12}
                bg={C.surface}
                rounded={8}
              >
                <button
                  w={120}
                  bg={C.accent}
                  cursor="pointer"
                  hover:bg={C.accentDim}
                >
                  button text
                </button>
              </view>
            </view>

            <view
              display="flex"
              flexDir="col"
              gap={8}
              flex={1}
              minW={200}
              p={16}
              bg={C.surface2}
              rounded={12}
              border={1}
              borderColor={C.border}
            >
              <view display="flex" flexDir="col" gap={4}>
                <text fontSize={13} fontWeight={600} color={C.accentHi}>
                  With Padding
                </text>
                <text fontSize={12} color={C.textMuted}>
                  px: 12 | py: 6
                </text>
              </view>
              <view
                display="flex"
                items="center"
                justify="center"
                p={12}
                bg={C.surface}
                rounded={8}
              >
                <button
                  w={120}
                  px={12}
                  py={6}
                  bg={C.accent}
                  cursor="pointer"
                  hover:bg={C.accentDim}
                >
                  button text
                </button>
              </view>
            </view>

            <view
              display="flex"
              flexDir="col"
              gap={8}
              flex={1}
              minW={200}
              p={16}
              bg={C.surface2}
              rounded={12}
              border={1}
              borderColor={C.border}
            >
              <view display="flex" flexDir="col" gap={4}>
                <text fontSize={13} fontWeight={600} color={C.accentHi}>
                  Uniform Padding
                </text>
                <text fontSize={12} color={C.textMuted}>
                  padding: 12
                </text>
              </view>
              <view
                display="flex"
                items="center"
                justify="center"
                p={12}
                bg={C.surface}
                rounded={8}
              >
                <button
                  w={120}
                  p={12}
                  bg={C.accent}
                  cursor="pointer"
                  hover:bg={C.accentDim}
                >
                  button text
                </button>
              </view>
            </view>

            <view
              display="flex"
              flexDir="col"
              gap={8}
              flex={1}
              minW={200}
              p={16}
              bg={C.surface2}
              rounded={12}
              border={1}
              borderColor={C.border}
            >
              <view display="flex" flexDir="col" gap={4}>
                <text fontSize={13} fontWeight={600} color={C.accentHi}>
                  Rounded
                </text>
                <text fontSize={12} color={C.textMuted}>
                  rounded: 8 | px: 12 | py: 6
                </text>
              </view>
              <view
                display="flex"
                items="center"
                justify="center"
                p={12}
                bg={C.surface}
                rounded={8}
              >
                <button
                  w={120}
                  rounded={8}
                  px={12}
                  py={6}
                  bg={C.accent}
                  cursor="pointer"
                  hover:bg={C.accentDim}
                >
                  button text
                </button>
              </view>
            </view>

            <view
              display="flex"
              flexDir="col"
              gap={8}
              flex={1}
              minW={200}
              p={16}
              bg={C.surface2}
              rounded={12}
              border={1}
              borderColor={C.border}
            >
              <view display="flex" flexDir="col" gap={4}>
                <text fontSize={13} fontWeight={600} color={C.accentHi}>
                  Flex Centered
                </text>
                <text fontSize={12} color={C.textMuted}>
                  flex-centered | px: 12 | py: 6 | rounded: 8
                </text>
              </view>
              <view
                display="flex"
                items="center"
                justify="center"
                p={12}
                bg={C.surface}
                rounded={8}
              >
                <button
                  display="flex"
                  flexDir="row"
                  justify="center"
                  w={120}
                  rounded={8}
                  px={12}
                  py={6}
                  bg={C.accent}
                  cursor="pointer"
                  hover:bg={C.accentDim}
                >
                  button text
                </button>
              </view>
            </view>
          </view>
        </view>

        <Divider />

        <view display="flex" flexDir="col" gap={12}>
          <text fontSize={14} fontWeight={700} color={C.text}>
            Checkboxes
          </text>
          <view
            display="flex"
            flexDir="col"
            p={16}
            gap={14}
            bg={C.surface2}
            rounded={8}
            border={1}
            borderColor={C.border}
          >
            <view display="flex" items="center" gap={12}>
              <checkbox
                checked={noRounding}
                onChange={setNoRounding}
                bg={C.accent}
                borderColor={noRounding ? C.accent : C.border}
                color="#ffffff"
                w={20}
                h={20}
                hover:opacity={0.9}
              />
              <text fontSize={14} color={C.text}>
                Square checkbox{noRounding ? ' [selected]' : ''}
              </text>
            </view>
            <view display="flex" items="center" gap={12}>
              <checkbox
                checked={rounded}
                onChange={setRounded}
                bg={C.success}
                borderColor={rounded ? C.success : C.border}
                color="#08110a"
                rounded={4}
                w={20}
                h={20}
              />
              <text fontSize={14} color={C.text}>
                Rounded checkbox{rounded ? ' [selected]' : ''}
              </text>
            </view>
            <view display="flex" items="center" gap={12}>
              <checkbox
                checked={circle}
                onChange={setCircle}
                bg={C.warning}
                borderColor={circle ? C.warning : C.border}
                color="#1b1104"
                rounded={10}
                w={20}
                h={20}
              />
              <text fontSize={14} color={C.text}>
                Circular checkbox{circle ? ' [selected]' : ''}
              </text>
            </view>
          </view>
        </view>

        <Divider />

        <view display="flex" flexDir="col" gap={12}>
          <text fontSize={14} fontWeight={700} color={C.text}>
            Deep nesting (6 levels)
          </text>
          <view
            p={16}
            bg={C.surface2}
            rounded={8}
            border={1}
            borderColor={C.border}
          >
            <view
              p={14}
              bg={C.surface3}
              rounded={8}
              border={1}
              borderColor={C.borderHi}
            >
              <view
                p={12}
                bg={C.surface4}
                rounded={8}
                border={1}
                borderColor={C.primaryDim}
              >
                <view
                  p={10}
                  bg={C.accentDark}
                  rounded={8}
                  border={1}
                  borderColor={C.accent}
                >
                  <view
                    p={8}
                    bg={C.accentDim}
                    rounded={8}
                    border={1}
                    borderColor={C.accentHi}
                  >
                    <view
                      p={6}
                      bg={C.accent}
                      rounded={4}
                      display="flex"
                      items="center"
                      justify="center"
                    >
                      <text fontSize={12} fontWeight={700} color="#fff">
                        6 levels
                      </text>
                    </view>
                  </view>
                </view>
              </view>
            </view>
          </view>
        </view>

        <Divider />

        <view display="flex" flexDir="col" gap={10}>
          <view display="flex" flexDir="row" items="center" gap={8}>
            <text fontSize={14} fontWeight={700} color={C.text}>
              Cursor kinds
            </text>
          </view>
          <view display="flex" flexDir="row" gap={8}>
            {(
              [
                'default',
                'pointer',
                'text',
                'crosshair',
                'not-allowed',
                'grab',
              ] as const
            ).map((cur) => (
              <view
                key={cur}
                px={14}
                py={10}
                bg={C.surface2}
                hover:bg={C.surface3}
                active:bg={C.surface4}
                rounded={8}
                border={1}
                borderColor={C.border}
                hover:borderColor={C.accentHi}
                cursor={cur}
              >
                <text fontSize={12} color={C.textDim} hover:color={C.text}>
                  {cur}
                </text>
              </view>
            ))}
          </view>
        </view>

        <Divider />

        <view display="flex" flexDir="col" gap={10}>
          <view display="flex" flexDir="row" items="center" gap={12}>
            <text fontSize={14} fontWeight={700} color={C.text}>
              visible prop
            </text>
            <button
              onClick={() => setShowHidden((s) => !s)}
              px={14}
              py={6}
              bg={showHidden ? C.accentDim : C.surface3}
              hover:bg={showHidden ? C.accent : C.surface4}
              rounded={8}
              border={1}
              borderColor={showHidden ? C.accent : C.border}
              cursor="pointer"
            >
              <text
                fontSize={12}
                fontWeight={600}
                color={showHidden ? C.accentHi : C.textMuted}
              >
                {showHidden ? 'Hide it' : 'Reveal it'}
              </text>
            </button>
          </view>
          <view
            visibility={showHidden ? 'visible' : 'hidden'}
            p={14}
            bg={C.accentDark}
            rounded={8}
            border={1}
            borderColor={C.accent}
          >
            <text fontSize={14} color={C.accentHi} fontWeight={600}>
              👁 Now you see me! (visibility)
            </text>
          </view>
          <view
            visibility={showHidden ? 'hidden' : 'visible'}
            p={14}
            bg={C.surface2}
            rounded={8}
            border={1}
            borderColor={C.border}
          >
            <text fontSize={14} color={C.textMuted}>
              Click the button to toggle visibility. It should appear above this
              text.
            </text>
          </view>
        </view>

        <view display="flex" flexDir="col" gap={10}>
          <view display="flex" flexDir="row" items="center" gap={12}>
            <text fontSize={14} fontWeight={700} color={C.text}>
              display prop (fallback)
            </text>
            <button
              onClick={() => setShowHidden((s) => !s)}
              px={14}
              py={6}
              bg={showHidden ? C.accentDim : C.surface3}
              hover:bg={showHidden ? C.accent : C.surface4}
              rounded={8}
              border={1}
              borderColor={showHidden ? C.accent : C.border}
              cursor="pointer"
            >
              <text
                fontSize={12}
                fontWeight={600}
                color={showHidden ? C.accentHi : C.textMuted}
              >
                {showHidden ? 'Hide it' : 'Show it'}
              </text>
            </button>
          </view>
          <view
            display={showHidden ? 'flex' : 'none'}
            p={14}
            bg={C.primaryDark}
            rounded={8}
            border={1}
            borderColor={C.primary}
          >
            <text fontSize={14} color={C.primaryHi} fontWeight={600}>
              👁 Now you see me via display!
            </text>
          </view>
          <view
            display={showHidden ? 'none' : 'flex'}
            p={14}
            bg={C.surface2}
            rounded={8}
            border={1}
            borderColor={C.border}
          >
            <text fontSize={14} color={C.textMuted}>
              Click the button to toggle with display. It should replace this
              text.
            </text>
          </view>
        </view>
      </view>
    </view>
  );
}
